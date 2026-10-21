use std::env;
use std::path::PathBuf;

pub(super) fn data_root() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("LEOCARD_SAKURA_DIR") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    base.map(|base| base.join("leocard").join("sakura-frp"))
        .ok_or_else(|| "无法确定 SakuraFrp 数据目录".to_owned())
}
