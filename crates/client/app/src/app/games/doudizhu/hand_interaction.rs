use super::DoudizhuUiState;
use super::cards::{HandCardSelectionOverlay, HandCardSlot};
use crate::app::presentation::{drag_preview_color, update_drag_selection};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::CardDragSelection;

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    client: Option<Res<ClientResource>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(&PickingInteraction, &RelativeCursorPosition, &HandCardSlot)>,
) {
    let Some(game) = client.as_deref().and_then(|c| c.0.model().doudizhu_game()) else {
        return;
    };
    if game.result.is_some()
        || game_ui.guide_open
        || game.players.iter().any(|p| p.id == game.you && p.auto_play)
    {
        drag.active = false;
        return;
    }
    let pressed = cards
        .iter()
        .find(|(interaction, _, _)| **interaction == PickingInteraction::Pressed)
        .map(|(_, _, slot)| (slot.index, slot.card));
    let hovered = cards
        .iter()
        .filter(|(_, cursor, _)| cursor.cursor_over())
        .map(|(_, _, slot)| slot.index)
        .max();
    ui.dirty |= update_drag_selection(
        &mouse,
        &mut drag,
        &mut game_ui.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
}

pub(super) fn sync_card_drag_preview(
    drag: Res<CardDragSelection>,
    mut overlays: Query<(&HandCardSelectionOverlay, &mut BackgroundColor)>,
) {
    if !drag.is_changed() {
        return;
    }
    for (overlay, mut color) in &mut overlays {
        color.0 = drag_preview_color(&drag, overlay.index);
    }
}
