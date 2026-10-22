use super::paths::data_root;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::time::{self, SystemTime};

#[derive(Clone, Deserialize, Serialize)]
pub struct SavedAccount {
    pub name: String,
    pub avatar_url: String,
    pub avatar_png: Option<Vec<u8>>,
}

/// Kept separate from game profiles and never serialized into network messages.
#[derive(Clone, Deserialize, Serialize)]
pub struct SavedLogin {
    pub token: String,
    pub account: SavedAccount,
}

pub struct LoginStore {
    path: PathBuf,
}

impl LoginStore {
    pub fn open() -> Result<Self, String> {
        Ok(Self {
            path: data_root()?.join("login.json"),
        })
    }

    pub fn load(&self) -> Result<Option<SavedLogin>, String> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("无法读取 SakuraFrp 登录信息".to_owned()),
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| "SakuraFrp 登录信息损坏，请重新登录".to_owned())
    }

    pub fn save(&self, login: &SavedLogin) -> Result<(), String> {
        let bytes = serde_json::to_vec(login).map_err(|_| "无法保存 SakuraFrp 登录信息")?;
        let parent = self.path.parent().ok_or("SakuraFrp 登录信息路径无效")?;
        fs::create_dir_all(parent).map_err(|_| "无法创建 SakuraFrp 登录信息目录")?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let nonce = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map_err(|_| "无法生成 SakuraFrp 登录信息临时文件")?
            .as_nanos();
        let temporary = self.path.with_extension(format!("{nonce}.tmp"));
        let result = (|| {
            let mut file = options
                .open(&temporary)
                .map_err(|_| "无法写入 SakuraFrp 登录信息")?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| "无法写入 SakuraFrp 登录信息")?;
            fs::rename(&temporary, &self.path).map_err(|_| "无法更新 SakuraFrp 登录信息")
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result.map_err(str::to_owned)
    }

    pub fn forget(&self) -> Result<(), String> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("无法清除 SakuraFrp 访问密钥".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn remembered_login_can_be_replaced_and_forgotten_without_leaving_credentials() {
        let nonce = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = env::temp_dir().join(format!("leocard-sakura-login-{nonce}"));
        let store = LoginStore {
            path: directory.join("login.json"),
        };
        assert!(store.load().unwrap().is_none());
        let mut saved = SavedLogin {
            token: "first-test-token".to_owned(),
            account: SavedAccount {
                name: "测试用户".to_owned(),
                avatar_url: "https://example.test/avatar".to_owned(),
                avatar_png: Some(vec![1, 2, 3]),
            },
        };
        store.save(&saved).unwrap();
        saved.token = "replacement-test-token".to_owned();
        store.save(&saved).unwrap();
        let restored = store.load().unwrap().unwrap();
        assert_eq!(restored.token, saved.token);
        assert_eq!(restored.account.name, saved.account.name);
        assert_eq!(restored.account.avatar_png, saved.account.avatar_png);
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&store.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        store.forget().unwrap();
        store.forget().unwrap();
        assert!(store.load().unwrap().is_none());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir(directory).unwrap();
    }
}
