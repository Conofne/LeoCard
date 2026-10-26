use super::DoudizhuUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_doudizhu::{Action, RuleSet, hint};
use leocard_protocol::DoudizhuCommand;

#[derive(Clone)]
pub(crate) enum DoudizhuUiAction {
    UpdateRules(RuleSet),
    ToggleCard,
    Clear,
    Hint,
    Play,
    Act(Action),
    ToggleGuide,
    ToggleCounter,
    ToggleCounterDrawer,
}

impl DomainUiAction for DoudizhuUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        if let UiAction::Doudizhu(action) = action {
            Some(action)
        } else {
            None
        }
    }

    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard)
    }
}

#[derive(SystemParam)]
pub(super) struct ActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, DoudizhuUiState>,
}

pub(super) fn dispatch_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ActionContext,
) {
    dispatch_domain_actions::<DoudizhuUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ActionContext<'_>> for DoudizhuUiAction {
    fn handle(&self, context: &mut ActionContext<'_>) {
        match self {
            Self::UpdateRules(rules) => send_game_command(
                &mut context.client,
                DoudizhuCommand::UpdateRules { rules: *rules },
            ),
            Self::Act(action) => {
                send_game_command(&mut context.client, DoudizhuCommand::Act(action.clone()))
            }
            Self::ToggleCard => {}
            Self::Clear => context.ui.selected.clear(),
            Self::Play => send_game_command(
                &mut context.client,
                DoudizhuCommand::Act(Action::Play(context.ui.selected.iter().copied().collect())),
            ),
            Self::Hint => {
                if let Some(game) = context
                    .client
                    .as_deref()
                    .and_then(|c| c.0.model().doudizhu_game())
                {
                    context.ui.selected = hint(
                        &game.your_hand,
                        game.table.as_ref().map(|(_, p)| p),
                        game.rules.mode,
                    )
                    .map(|p| p.cards.into_iter().collect())
                    .unwrap_or_default();
                }
            }
            Self::ToggleGuide => context.ui.guide_open = !context.ui.guide_open,
            Self::ToggleCounter => context.ui.counter.enabled = !context.ui.counter.enabled,
            Self::ToggleCounterDrawer => {
                context.ui.counter.drawer_open = !context.ui.counter.drawer_open
            }
        }
    }
}
