use super::actions::{SakuraInput, SakuraUiAction};
use super::input::SakuraScroll;
use super::state::SakuraState;
use crate::app::presentation::{
    MUTED, Scrollbar, ScrollbarSkin, TEXT, TextInput, UiComposer, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
    add_page_back_title,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_sakura_frp::{Tunnel, TunnelType};

pub(crate) struct SakuraPage<'a> {
    pub state: &'a SakuraState,
    pub assets: &'a UiAssets,
}

impl SakuraPage<'_> {
    pub fn render(&self, commands: &mut UiComposer, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let page = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1220),
                min_height: px(0),
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        add_page_back_title(
            commands,
            page,
            "SakuraFrp 内网穿透",
            UiAction::Sakura(SakuraUiAction::Close),
            self.assets,
        );
        add_text(commands, page, &self.state.status, 14.0, MUTED, self.assets);
        if let Some(warning) = &self.state.warning {
            add_text(
                commands,
                page,
                warning,
                13.0,
                Color::srgb(1.0, 0.77, 0.46),
                self.assets,
            );
        }
        if self.state.client.is_none() {
            self.render_login(commands, page);
            return;
        }
        self.render_account(commands, page);
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                column_gap: px(18),
                ..default()
            },
            None,
        );
        self.render_tunnels(commands, body);
        self.render_form(commands, body);
    }

    fn render_login(&self, commands: &mut UiComposer, page: Entity) {
        let panel = panel(commands, page, 1.0);
        add_text(commands, panel, "访问密钥登录", 24.0, TEXT, self.assets);
        add_text(
            commands,
            panel,
            "在 SakuraFrp 控制台复制访问密钥，再点击下方按钮。密钥只保留在本次 LeoCard 运行期间。",
            15.0,
            MUTED,
            self.assets,
        );
        self.button(
            commands,
            panel,
            "从剪贴板登录",
            SakuraUiAction::LoginFromClipboard,
            false,
            self.state.busy,
            px(190),
        );
    }

    fn render_account(&self, commands: &mut UiComposer, page: Entity) {
        let row = row(commands, page, 12.0);
        if let Some(user) = &self.state.user {
            let group = user
                .group
                .as_ref()
                .map_or("普通用户", |group| group.name.as_str());
            add_text(
                commands,
                row,
                format!("{}  UID {}  {}", user.name, user.id, group),
                15.0,
                TEXT,
                self.assets,
            );
        }
        self.button(
            commands,
            row,
            "刷新状态",
            SakuraUiAction::Refresh,
            false,
            self.state.busy,
            px(112),
        );
        self.button(
            commands,
            row,
            "退出账号",
            SakuraUiAction::Logout,
            false,
            self.state.busy,
            px(112),
        );
    }

    fn render_tunnels(&self, commands: &mut UiComposer, body: Entity) {
        let panel = panel(commands, body, 0.58);
        add_text(
            commands,
            panel,
            format!("我的隧道  {}", self.state.tunnels.len()),
            22.0,
            TEXT,
            self.assets,
        );
        let list = scroll_list(
            commands,
            panel,
            "sakura-tunnels",
            SakuraScroll::Tunnels,
            self.state.tunnel_scroll_y,
        );
        if self.state.tunnels.is_empty() {
            add_text(
                commands,
                list,
                "还没有隧道。先在右侧选择协议、节点和本地端口。",
                15.0,
                MUTED,
                self.assets,
            );
        }
        for tunnel in &self.state.tunnels {
            self.render_tunnel(commands, list, tunnel);
        }
    }

    fn render_tunnel(&self, commands: &mut UiComposer, list: Entity, tunnel: &Tunnel) {
        let card = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                ..default()
            },
            Some(Color::srgba(0.16, 0.18, 0.23, 0.9)),
        );
        add_text(
            commands,
            card,
            format!(
                "{}  #{}  {}",
                tunnel.name,
                tunnel.id,
                tunnel.kind.to_ascii_uppercase()
            ),
            17.0,
            TEXT,
            self.assets,
        );
        let local = format!("{}:{}", tunnel.local_ip, tunnel.local_port);
        add_text(
            commands,
            card,
            format!("本地 {local}    公网 {}", tunnel.address(&self.state.nodes)),
            14.0,
            MUTED,
            self.assets,
        );
        let node = self
            .state
            .nodes
            .iter()
            .find(|node| node.id == tunnel.node)
            .map_or("未知节点", |node| node.name.as_str());
        let status = if tunnel.status != 0 {
            tunnel.status_reason.as_deref().unwrap_or("服务端已禁用")
        } else if tunnel.online {
            "服务端在线"
        } else {
            "服务端离线"
        };
        let local_status = if self.state.running.contains_key(&tunnel.id) {
            "本机运行中"
        } else {
            "本机未启动"
        };
        add_text(
            commands,
            card,
            format!("{node}  {status}  {local_status}"),
            13.0,
            MUTED,
            self.assets,
        );
        if !tunnel.note.is_empty() {
            add_text(commands, card, &tunnel.note, 12.0, MUTED, self.assets);
        }
        let controls = row(commands, card, 8.0);
        if self.state.running.contains_key(&tunnel.id) {
            self.button(
                commands,
                controls,
                "停止",
                SakuraUiAction::Stop(tunnel.id),
                false,
                false,
                px(90),
            );
        } else {
            self.button(
                commands,
                controls,
                "启动",
                SakuraUiAction::Start(tunnel.id),
                false,
                self.state.busy || tunnel.status != 0,
                px(90),
            );
        }
        if self.state.pending_delete == Some(tunnel.id) {
            self.button(
                commands,
                controls,
                "确认删除",
                SakuraUiAction::ConfirmDelete(tunnel.id),
                false,
                self.state.busy,
                px(115),
            );
            self.button(
                commands,
                controls,
                "取消",
                SakuraUiAction::CancelDelete,
                false,
                false,
                px(80),
            );
        } else {
            self.button(
                commands,
                controls,
                "删除",
                SakuraUiAction::AskDelete(tunnel.id),
                false,
                self.state.busy,
                px(90),
            );
        }
    }

    fn render_form(&self, commands: &mut UiComposer, body: Entity) {
        let panel = panel(commands, body, 0.42);
        let form = scroll_list(
            commands,
            panel,
            "sakura-form",
            SakuraScroll::Form,
            self.state.form_scroll_y,
        );
        add_text(commands, form, "创建隧道", 22.0, TEXT, self.assets);
        add_text(
            commands,
            form,
            "适用于任何使用本机端口的游戏或服务。需要 TCP 和 UDP 时分别创建隧道。",
            13.0,
            MUTED,
            self.assets,
        );
        self.field(commands, form, SakuraInput::Name);
        let kinds = row(commands, form, 6.0);
        for kind in [
            TunnelType::Tcp,
            TunnelType::Udp,
            TunnelType::Http,
            TunnelType::Https,
        ] {
            self.button(
                commands,
                kinds,
                kind.label(),
                SakuraUiAction::SelectKind(kind),
                self.state.kind == kind,
                false,
                px(75),
            );
        }
        self.field(commands, form, SakuraInput::LocalIp);
        self.field(commands, form, SakuraInput::LocalPort);
        self.field(commands, form, SakuraInput::Remote);
        self.button(
            commands,
            form,
            if self.state.public_game {
                "公开游戏服务器  是"
            } else {
                "公开游戏服务器  否"
            },
            SakuraUiAction::TogglePublicGame,
            self.state.public_game,
            false,
            percent(100),
        );
        add_text(
            commands,
            form,
            "公开游戏服务器只能使用专用节点。私人联机及其他应用可使用普通节点。",
            12.0,
            MUTED,
            self.assets,
        );
        add_text(commands, form, "选择节点", 16.0, TEXT, self.assets);
        self.render_nodes(commands, form);
        self.button(
            commands,
            form,
            "创建隧道",
            SakuraUiAction::Create,
            true,
            self.state.busy || self.state.node.is_none(),
            percent(100),
        );
        add_text(
            commands,
            form,
            "启动时自动下载并校验官方 frpc。关闭 LeoCard 或退出账号后本机隧道停止。",
            12.0,
            MUTED,
            self.assets,
        );
    }

    fn render_nodes(&self, commands: &mut UiComposer, form: Entity) {
        let container = spawn_node(
            commands,
            form,
            Node {
                width: percent(100),
                height: px(168),
                min_height: px(0),
                ..default()
            },
            None,
        );
        let list = scroll_list(
            commands,
            container,
            "sakura-nodes",
            SakuraScroll::Nodes,
            self.state.node_scroll_y,
        );
        let mut count = 0;
        for node in self
            .state
            .nodes
            .iter()
            .filter(|node| self.state.node_compatible(node))
        {
            count += 1;
            let label = format!("{}  #{}  VIP {}", node.name, node.id, node.vip);
            self.button(
                commands,
                list,
                &label,
                SakuraUiAction::SelectNode(node.id),
                self.state.node == Some(node.id),
                false,
                percent(100),
            );
        }
        if count == 0 {
            add_text(
                commands,
                list,
                "没有符合当前协议和用途的可用节点",
                13.0,
                MUTED,
                self.assets,
            );
        }
    }

    fn field(&self, commands: &mut UiComposer, parent: Entity, kind: SakuraInput) {
        let (label, key, value, placeholder) = match kind {
            SakuraInput::Name => (
                "隧道名称",
                "sakura.name",
                self.state.name.as_str(),
                "例如 我的游戏",
            ),
            SakuraInput::LocalIp => (
                "本地 IP",
                "sakura.local_ip",
                self.state.local_ip.as_str(),
                "127.0.0.1",
            ),
            SakuraInput::LocalPort => (
                "本地端口",
                "sakura.local_port",
                self.state.local_port.as_str(),
                "1 到 65535",
            ),
            SakuraInput::Remote => (
                "远程端口或域名",
                "sakura.remote",
                self.state.remote.as_str(),
                "TCP/UDP 可留空随机分配",
            ),
        };
        add_text(commands, parent, label, 14.0, MUTED, self.assets);
        let mut input = TextInput::new(key, value);
        input.placeholder = placeholder;
        input.max_characters = 100;
        let editor = input.spawn(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(40),
                padding: UiRect::axes(px(12), px(8)),
                ..default()
            },
            self.assets,
        );
        commands.entity(editor).insert(kind);
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "button state and size stay explicit in the page"
    )]
    fn button(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        label: &str,
        action: SakuraUiAction,
        selected: bool,
        disabled: bool,
        width: Val,
    ) {
        if disabled {
            add_cozy_disabled_button(commands, parent, label, self.assets, width, 40.0);
        } else {
            add_cozy_button_variant(
                commands,
                parent,
                label,
                UiAction::Sakura(action),
                self.assets,
                width,
                40.0,
                if selected {
                    CozyButtonVariant::Primary
                } else {
                    CozyButtonVariant::Neutral
                },
            );
        }
    }
}

fn panel(commands: &mut UiComposer, parent: Entity, grow: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(0),
            flex_basis: px(0),
            flex_grow: grow,
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        Some(Color::srgba(0.10, 0.12, 0.16, 0.85)),
    )
}

fn row(commands: &mut UiComposer, parent: Entity, gap: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(gap),
            ..default()
        },
        None,
    )
}

fn scroll_list(
    commands: &mut UiComposer,
    parent: Entity,
    key: &str,
    kind: SakuraScroll,
    scroll_y: f32,
) -> Entity {
    let viewport = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(8),
            ..default()
        },
        None,
    );
    let list = commands.keyed_child(
        viewport,
        key,
        (
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            kind,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, scroll_y)),
        ),
    );
    Scrollbar::spawn(
        commands,
        viewport,
        list,
        ScrollbarSkin::Colored(Color::srgb(0.70, 0.75, 0.87)),
    );
    list
}
