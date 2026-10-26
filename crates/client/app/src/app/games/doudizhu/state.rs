use crate::app::presentation::CardCounterWindowState;
use bevy::prelude::*;
use leocard_doudizhu::Card;
use leocard_protocol::{DoudizhuSnapshot, MatchId};
use leocard_ui::{DraggableWindowOwner, DraggableWindowState};
use std::collections::HashSet;

#[derive(Default, Resource)]
pub(crate) struct DoudizhuUiState {
    pub selected: HashSet<Card>,
    pub guide_open: bool,
    pub counter: CardCounterWindowState,
    match_id: Option<MatchId>,
}

impl DoudizhuUiState {
    pub fn reconcile(&mut self, game: &DoudizhuSnapshot) {
        if self.match_id != Some(game.match_id) {
            self.selected.clear();
            self.match_id = Some(game.match_id);
        }
        self.selected.retain(|card| game.your_hand.contains(card));
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.match_id = None;
        self.guide_open = false;
        self.counter.active = false;
    }
}

impl DraggableWindowOwner for DoudizhuUiState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.counter.drag
    }
}
