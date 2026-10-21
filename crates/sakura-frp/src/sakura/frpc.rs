use super::api::SakuraClient;
use super::paths::data_root;
use md5::{Digest, Md5};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Deserialize)]
struct ClientRelease {
    ver: String,
    archs: HashMap<String, ClientArtifact>,
}

#[derive(Deserialize)]
struct ClientArtifact {
    url: String,
    hash: String,
}

pub struct RunningTunnel {
    pub id: u64,
    child: Child,
    config_path: PathBuf,
    pub log_path: PathBuf,
}

impl RunningTunnel {
    pub fn running(&mut self) -> Result<bool, String> {
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|error| format!("检查 frpc 状态失败：{error}"))
    }
}

impl Drop for RunningTunnel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.config_path);
    }
}

impl SakuraClient {
    pub fn start_tunnel(
        &self,
        id: u64,
        mut progress: impl FnMut(String),
    ) -> Result<RunningTunnel, String> {
        progress("获取 SakuraFrp 客户端信息".to_owned());
        let releases: HashMap<String, ClientRelease> = self.public_json("/system/clients")?;
        let release = releases.get("frpc").ok_or("官方目录没有 frpc 下载信息")?;
        let artifact = release
            .archs
            .get(platform_arch()?)
            .ok_or("官方未提供当前系统架构的 frpc")?;
        let url = reqwest::Url::parse(&artifact.url)
            .map_err(|error| format!("frpc 下载地址无效：{error}"))?;
        if url.scheme() != "https" || url.host_str() != Some("nya.globalslb.net") {
            return Err("frpc 下载地址不是官方 HTTPS 地址".to_owned());
        }
        let version = url
            .path_segments()
            .and_then(|mut parts| {
                parts.position(|part| part == "frpc")?;
                parts.next().map(str::to_owned)
            })
            .unwrap_or_else(|| release.ver.clone());
        if version.is_empty()
            || !version
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err("frpc 版本名称无效".to_owned());
        }
        let expected = artifact.hash.to_ascii_lowercase();
        if expected.len() != 32 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("官方 frpc 校验值无效".to_owned());
        }
        let root = data_root()?;
        let binary = root
            .join("bin")
            .join(&version)
            .join(if cfg!(target_os = "windows") {
                "frpc.exe"
            } else {
                "frpc"
            });
        progress(format!("准备 frpc {version}"));
        self.download_binary(url.as_str(), &binary, &expected)?;
        progress(format!("获取隧道 {id} 的运行配置"));
        let config = self.config(id, &version)?;
        let config_path = root.join("config").join(format!("tunnel-{id}.ini"));
        write_private(&config_path, config.as_bytes())?;
        let log_path = root.join("logs").join(format!("tunnel-{id}.log"));
        fs::create_dir_all(log_path.parent().ok_or("日志目录无效")?)
            .map_err(|error| error.to_string())?;
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("无法创建 frpc 日志：{error}"))?;
        let errors = log.try_clone().map_err(|error| error.to_string())?;
        let mut command = Command::new(&binary);
        command
            .arg("-c")
            .arg(&config_path)
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(errors));
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                let _ = fs::remove_file(&config_path);
                return Err(format!("启动 frpc 失败：{error}"));
            }
        };
        Ok(RunningTunnel {
            id,
            child,
            config_path,
            log_path,
        })
    }

    fn download_binary(&self, url: &str, target: &Path, expected: &str) -> Result<(), String> {
        if target.is_file() && file_hash(target)? == expected {
            return Ok(());
        }
        fs::create_dir_all(target.parent().ok_or("frpc 下载目录无效")?)
            .map_err(|error| error.to_string())?;
        let temporary = target.with_extension("part");
        let result = (|| {
            let mut response = self
                .http
                .get(url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|error| format!("下载 frpc 失败：{error}"))?;
            let mut output = File::create(&temporary).map_err(|error| error.to_string())?;
            let mut hasher = Md5::new();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let count = response
                    .read(&mut buffer)
                    .map_err(|error| error.to_string())?;
                if count == 0 {
                    break;
                }
                output
                    .write_all(&buffer[..count])
                    .map_err(|error| error.to_string())?;
                hasher.update(&buffer[..count]);
            }
            output.flush().map_err(|error| error.to_string())?;
            if format!("{:x}", hasher.finalize()) != expected {
                return Err("frpc 下载文件校验失败".to_owned());
            }
            if target.exists() {
                fs::remove_file(target).map_err(|error| error.to_string())?;
            }
            fs::rename(&temporary, target).map_err(|error| error.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(target, fs::Permissions::from_mode(0o700))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Md5::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("配置目录无效")?).map_err(|error| error.to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(contents).map_err(|error| error.to_string())
}

fn platform_arch() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("windows_amd64"),
        ("windows", "x86") => Ok("windows_386"),
        ("windows", "aarch64") => Ok("windows_arm64"),
        ("linux", "x86_64") => Ok("linux_amd64"),
        ("linux", "x86") => Ok("linux_386"),
        ("linux", "aarch64") => Ok("linux_arm64"),
        ("linux", "arm") => Ok("linux_armv7"),
        ("macos", "x86_64") => Ok("darwin_amd64"),
        ("macos", "aarch64") => Ok("darwin_arm64"),
        _ => Err("当前系统架构不支持内置 frpc".to_owned()),
    }
}
