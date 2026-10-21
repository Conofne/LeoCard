use super::actions::{dispatch_sakura_actions, refresh, sync_sakura_inputs};
use super::input::{remember_sakura_scroll, scroll_sakura_lists};
use super::state::{SakuraState, TaskMessage, TaskOutcome};
use crate::app::presentation::TextInputSet;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{UiActionSet, UiState};
use bevy::prelude::*;
use bevy_clipboard::Clipboard;
use std::time::{Duration, Instant};

pub(crate) struct SakuraFrpPlugin;

impl Plugin for SakuraFrpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Clipboard>()
            .init_resource::<SakuraState>()
            .add_systems(Update, dispatch_sakura_actions.in_set(UiActionSet))
            .add_systems(Update, scroll_sakura_lists.in_set(ClientUpdateSet::Input))
            .add_systems(
                Update,
                (
                    remember_sakura_scroll,
                    poll_sakura_tasks,
                    poll_sakura_processes,
                    refresh_periodically,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(PostUpdate, sync_sakura_inputs.in_set(TextInputSet::Publish));
    }
}

fn poll_sakura_tasks(mut state: ResMut<SakuraState>, mut ui: ResMut<UiState>) {
    let messages = {
        let Ok(mut queue) = state.messages.lock() else {
            return;
        };
        std::mem::take(&mut *queue)
    };
    if messages.is_empty() {
        return;
    }
    for message in messages {
        match message {
            TaskMessage::Finished(result) => {
                state.busy = false;
                match result {
                    Ok(TaskOutcome::LoggedIn(client, dashboard)) => {
                        let name = dashboard.user.name.clone();
                        state.client = Some(client);
                        state.apply_dashboard(dashboard);
                        state.status = format!("已登录 SakuraFrp：{name}");
                    }
                    Ok(TaskOutcome::Refreshed(dashboard)) => {
                        state.apply_dashboard(dashboard);
                        state.status = format!("已更新 {} 条隧道", state.tunnels.len());
                    }
                    Ok(TaskOutcome::Created(created, dashboard)) => {
                        state.status = format!("隧道 {} 已创建，ID {}", created.name, created.id);
                        state.warning = created.important_warning_for_ai;
                        if let Some(dashboard) = dashboard {
                            state.apply_dashboard(dashboard);
                        }
                        state.name.clear();
                        state.local_port.clear();
                        state.remote.clear();
                    }
                    Ok(TaskOutcome::Deleted(id, failed_to_disconnect, dashboard)) => {
                        state.running.remove(&id);
                        state.tunnels.retain(|tunnel| tunnel.id != id);
                        if let Some(dashboard) = dashboard {
                            state.apply_dashboard(dashboard);
                        }
                        state.status = if failed_to_disconnect {
                            format!("隧道 {id} 已删除，服务端未能立即踢下线")
                        } else {
                            format!("隧道 {id} 已删除")
                        };
                    }
                    Ok(TaskOutcome::Started(running)) => {
                        let id = running.id;
                        state.running.insert(id, running);
                        state.status = format!("隧道 {id} 已在本机启动");
                    }
                    Err(error) => state.status = error,
                }
            }
        }
    }
    ui.dirty = true;
}

fn poll_sakura_processes(mut state: ResMut<SakuraState>, mut ui: ResMut<UiState>) {
    let mut exited = Vec::new();
    state.running.retain(|id, process| {
        if process.running().unwrap_or(false) {
            return true;
        }
        exited.push((*id, process.log_path.clone()));
        false
    });
    if let Some((id, log)) = exited.last() {
        state.status = format!("隧道 {id} 的 frpc 已退出，请查看 {}", log.display());
        ui.dirty = true;
    }
}

fn refresh_periodically(mut state: ResMut<SakuraState>) {
    if !state.open || state.client.is_none() || state.busy || Instant::now() < state.next_refresh {
        return;
    }
    state.next_refresh = Instant::now() + Duration::from_secs(30);
    refresh(&mut state);
}
