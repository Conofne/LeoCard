use super::feedback::{DoudizhuFeedback, play_feedback};
use super::hand_interaction::{handle_card_drag_selection, sync_card_drag_preview};
use super::{DoudizhuUiState, actions::dispatch_actions};
use crate::app::presentation::drag_card_counter_window;
use crate::app::runtime::{ClientResource, ClientUpdateSet};
use crate::app::shell::{UiActionSet, UiState};
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_doudizhu::Phase;

pub(crate) struct DoudizhuPlugin;

impl Plugin for DoudizhuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DoudizhuUiState>()
            .init_resource::<DoudizhuFeedback>()
            .add_systems(Update, dispatch_actions.in_set(UiActionSet))
            .add_systems(Update, sync.in_set(ClientUpdateSet::Sync))
            .add_systems(Update, play_feedback.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                (
                    drag_card_counter_window::<DoudizhuUiState>,
                    handle_card_drag_selection,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                sync_card_drag_preview.in_set(ClientUpdateSet::Animate),
            );
    }
}

fn sync(
    client: Option<Res<ClientResource>>,
    economy: Res<PlayerEconomy>,
    mut state: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
) {
    let game = client.as_deref().and_then(|c| c.0.model().doudizhu_game());
    let active = game.is_some_and(|g| g.phase == Phase::Playing && !g.your_hand.is_empty())
        && economy.active(ItemId::DoudizhuCardCounter);
    if state.counter.active != active {
        state.counter.active = active;
        state.counter.grab = None;
        ui.dirty = true;
    }
}
