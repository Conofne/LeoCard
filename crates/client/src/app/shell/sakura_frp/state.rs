use bevy::prelude::*;
use leocard_sakura_frp::{
    CreatedTunnel, Dashboard, Node, RunningTunnel, SakuraClient, Tunnel, TunnelType, UserInfo,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(super) enum TaskOutcome {
    LoggedIn(SakuraClient, Dashboard),
    Refreshed(Dashboard),
    Created(CreatedTunnel, Option<Dashboard>),
    Deleted(u64, bool, Option<Dashboard>),
    Started(RunningTunnel),
}

pub(super) enum TaskMessage {
    Finished(Result<TaskOutcome, String>),
}

#[derive(Resource)]
pub(crate) struct SakuraState {
    pub open: bool,
    pub client: Option<SakuraClient>,
    pub user: Option<UserInfo>,
    pub nodes: Vec<Node>,
    pub tunnels: Vec<Tunnel>,
    pub running: HashMap<u64, RunningTunnel>,
    pub name: String,
    pub kind: TunnelType,
    pub node: Option<u64>,
    pub local_ip: String,
    pub local_port: String,
    pub remote: String,
    pub public_game: bool,
    pub pending_delete: Option<u64>,
    pub status: String,
    pub warning: Option<String>,
    pub busy: bool,
    pub next_refresh: Instant,
    pub tunnel_scroll_y: f32,
    pub form_scroll_y: f32,
    pub node_scroll_y: f32,
    pub(super) messages: Arc<Mutex<Vec<TaskMessage>>>,
}

impl Default for SakuraState {
    fn default() -> Self {
        Self {
            open: false,
            client: None,
            user: None,
            nodes: Vec::new(),
            tunnels: Vec::new(),
            running: HashMap::new(),
            name: String::new(),
            kind: TunnelType::Tcp,
            node: None,
            local_ip: "127.0.0.1".to_owned(),
            local_port: String::new(),
            remote: String::new(),
            public_game: false,
            pending_delete: None,
            status: "复制 SakuraFrp 访问密钥后登录".to_owned(),
            warning: None,
            busy: false,
            next_refresh: Instant::now() + Duration::from_secs(30),
            tunnel_scroll_y: 0.0,
            form_scroll_y: 0.0,
            node_scroll_y: 0.0,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SakuraState {
    pub fn apply_dashboard(&mut self, dashboard: Dashboard) {
        self.user = Some(dashboard.user);
        self.nodes = dashboard.nodes;
        self.tunnels = dashboard.tunnels;
        self.select_compatible_node();
        self.next_refresh = Instant::now() + Duration::from_secs(30);
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.nodes.iter().find(|node| Some(node.id) == self.node)
    }

    pub fn node_compatible(&self, node: &Node) -> bool {
        node.can_create()
            && (!self.public_game || node.allows_public_game())
            && (self.kind != TunnelType::Udp || node.supports_udp())
    }

    pub fn select_compatible_node(&mut self) {
        if self
            .selected_node()
            .is_some_and(|node| self.node_compatible(node))
        {
            return;
        }
        self.node = self
            .nodes
            .iter()
            .find(|node| self.node_compatible(node))
            .map(|node| node.id);
        self.node_scroll_y = 0.0;
    }
}
