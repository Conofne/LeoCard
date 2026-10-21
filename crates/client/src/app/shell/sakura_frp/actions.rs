use super::state::{SakuraState, TaskMessage, TaskOutcome};
use crate::app::shell::{DomainUiAction, PressedUiAction, UiAction, UiState};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy_clipboard::Clipboard;
use leocard_sakura_frp::{CreateTunnel, SakuraClient, TunnelType};
use std::thread;

#[derive(Clone)]
pub(crate) enum SakuraUiAction {
    Show,
    Close,
    LoginFromClipboard,
    Logout,
    Refresh,
    SelectKind(TunnelType),
    SelectNode(u64),
    TogglePublicGame,
    Create,
    Start(u64),
    Stop(u64),
    AskDelete(u64),
    CancelDelete,
    ConfirmDelete(u64),
}

impl DomainUiAction for SakuraUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Sakura(action) = action else {
            return None;
        };
        Some(action)
    }
}

pub(super) fn dispatch_sakura_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut state: ResMut<SakuraState>,
    mut clipboard: ResMut<Clipboard>,
    mut ui: ResMut<UiState>,
) {
    for pressed in actions.read() {
        let Some(action) = SakuraUiAction::extract(&pressed.0) else {
            continue;
        };
        ui.dirty = true;
        match action {
            SakuraUiAction::Show => {
                state.open = true;
                ui.shop.open = false;
                ui.achievements.open = false;
                ui.settings.open = false;
                ui.profile.open = false;
                if state.client.is_some() {
                    refresh(&mut state);
                }
            }
            SakuraUiAction::Close => state.open = false,
            SakuraUiAction::LoginFromClipboard => {
                let result = clipboard
                    .fetch_text()
                    .poll_result()
                    .ok_or("剪贴板暂时不可读取".to_owned())
                    .and_then(|result| result.map_err(|error| format!("读取剪贴板失败：{error}")));
                match result {
                    Ok(token) => {
                        start_task(&mut state, "验证 SakuraFrp 访问密钥", move || {
                            let client = SakuraClient::new(token)?;
                            let dashboard = client.dashboard()?;
                            Ok(TaskOutcome::LoggedIn(client, dashboard))
                        })
                    }
                    Err(error) => state.status = error,
                }
            }
            SakuraUiAction::Logout => {
                if !state.busy {
                    state.running.clear();
                    state.client = None;
                    state.user = None;
                    state.nodes.clear();
                    state.tunnels.clear();
                    state.status = "已退出 SakuraFrp，隧道已停止".to_owned();
                }
            }
            SakuraUiAction::Refresh => refresh(&mut state),
            SakuraUiAction::SelectKind(kind) => {
                state.kind = *kind;
                state.select_compatible_node();
            }
            SakuraUiAction::SelectNode(id) => {
                if state
                    .nodes
                    .iter()
                    .any(|node| node.id == *id && state.node_compatible(node))
                {
                    state.node = Some(*id);
                }
            }
            SakuraUiAction::TogglePublicGame => {
                state.public_game = !state.public_game;
                state.select_compatible_node();
            }
            SakuraUiAction::Create => {
                let Some(node) = state.selected_node().cloned() else {
                    state.status = "请先选择可用节点".to_owned();
                    continue;
                };
                let Ok(port) = state.local_port.trim().parse::<u16>() else {
                    state.status = "本地端口需为 1 到 65535".to_owned();
                    continue;
                };
                let request = CreateTunnel {
                    name: state.name.trim().to_owned(),
                    kind: state.kind,
                    node: node.id,
                    local_ip: state.local_ip.trim().to_owned(),
                    local_port: port,
                    remote: (!state.remote.trim().is_empty())
                        .then(|| state.remote.trim().to_owned()),
                    public_game: state.public_game,
                };
                if let Err(error) = request.validate(&node) {
                    state.status = error;
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                start_task(&mut state, "创建 SakuraFrp 隧道", move || {
                    let created = client.create_tunnel(&request, &node)?;
                    let dashboard = client.dashboard().ok();
                    Ok(TaskOutcome::Created(created, dashboard))
                });
            }
            SakuraUiAction::Start(id) => {
                if state.running.contains_key(id) {
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                let id = *id;
                start_task(&mut state, "启动内网穿透", move || {
                    client.start_tunnel(id, |_| {}).map(TaskOutcome::Started)
                });
            }
            SakuraUiAction::Stop(id) => {
                if state.running.remove(id).is_some() {
                    state.status = format!("隧道 {id} 已在本机停止");
                }
            }
            SakuraUiAction::AskDelete(id) => state.pending_delete = Some(*id),
            SakuraUiAction::CancelDelete => state.pending_delete = None,
            SakuraUiAction::ConfirmDelete(id) => {
                if state.pending_delete != Some(*id) {
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                let id = *id;
                state.pending_delete = None;
                start_task(&mut state, "删除 SakuraFrp 隧道", move || {
                    let failed_to_disconnect = client.delete_tunnel(id)?;
                    let dashboard = client.dashboard().ok();
                    Ok(TaskOutcome::Deleted(id, failed_to_disconnect, dashboard))
                });
            }
        }
    }
}

pub(super) fn refresh(state: &mut SakuraState) {
    let Some(client) = state.client.clone() else {
        return;
    };
    start_task(state, "刷新隧道状态", move || {
        client.dashboard().map(TaskOutcome::Refreshed)
    });
}

fn start_task(
    state: &mut SakuraState,
    title: &str,
    task: impl FnOnce() -> Result<TaskOutcome, String> + Send + 'static,
) {
    if state.busy {
        return;
    }
    state.busy = true;
    state.status = title.to_owned();
    let messages = state.messages.clone();
    thread::spawn(move || {
        let result = task();
        if let Ok(mut queue) = messages.lock() {
            queue.push(TaskMessage::Finished(result));
        }
    });
}

#[derive(Component, Clone, Copy)]
pub(super) enum SakuraInput {
    Name,
    LocalIp,
    LocalPort,
    Remote,
}

pub(super) fn sync_sakura_inputs(
    inputs: Query<(&SakuraInput, &EditableText)>,
    mut state: ResMut<SakuraState>,
    mut ui: ResMut<UiState>,
) {
    for (kind, input) in &inputs {
        let value = match kind {
            SakuraInput::Name => &mut state.name,
            SakuraInput::LocalIp => &mut state.local_ip,
            SakuraInput::LocalPort => &mut state.local_port,
            SakuraInput::Remote => &mut state.remote,
        };
        if input.value() != value.as_str() {
            *value = input.value().to_string();
            ui.dirty = true;
        }
    }
}
