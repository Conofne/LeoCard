# LeoCard

[![CI and Release](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml/badge.svg)](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml)

使用 Rust、Bevy 开发的跨平台局域网棋牌游戏。目前包含七鬼五二三、德州扑克、升级（拖拉机）、UNO 牌及变种、麻将和 MC 竞猜。

**本项目完全由 AI 开发，不保证代码质量和可维护性。**——当然，本人也会对质量做整体把关，当明显代码质量不佳时会使用 AI 进行大批量重构 (可以参考 [大幅重构记录](https://github.com/LeoDreamer2004/LeoCard/commit/96c51687c46f9fdf608d6e751f81da2259dce13f))。

运行当前联机客户端：

```bash
cargo run -p leocard-client
```

开发时客户端会从工作区根目录的 `assets` 加载素材。内嵌文件列表由 `crates/client/runtime-assets.txt` 管理。

## SakuraFrp 内网穿透

桌面端首页顶栏提供「内网穿透」。在 SakuraFrp 控制台复制访问密钥，点击「从剪贴板登录」，即可查看账号、节点和隧道信息。访问密钥仅保存在本次运行的内存中。可以创建、启动、停止和删除 TCP、UDP、HTTP、HTTPS 隧道；游戏服务器按实际使用的协议和本地端口配置，适用于 Minecraft 以外的游戏。若游戏同时使用 TCP 和 UDP，需分别创建隧道。

公开发布的游戏服务器只能使用 SakuraFrp 标记为允许公开游戏的节点，或名称包含「多线」的节点。创建时请开启「公开游戏服务器」选项；私人联机可以关闭。首次启动隧道时，LeoCard 从 SakuraFrp 官方地址下载 frpc 并校验文件。退出账号或关闭 LeoCard 会停止本机 frpc。数据默认保存在系统数据目录的 `leocard/sakura-frp`，也可用 `LEOCARD_SAKURA_DIR` 指定位置。Android 端暂不提供该入口。

自 0.7.0 版本起，本游戏支持 ARM64 的 Android 12 及以上客户端。

## 许可证

LeoCard 的原创代码以 [GPL v3.0](LICENSE) 发布。分发修改版时须按
GPL-3.0 提供相应源代码和许可证声明。第三方字体、图片、音频等素材继续使用各自的许可证，具体来源、署名和再分发要求见 [ASSETS.md](./assets/ASSETS.md)。
