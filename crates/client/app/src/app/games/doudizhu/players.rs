use super::cards::card_row;
use crate::app::presentation::{
    ACCENT, CardSize, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, add_player_portrait,
    add_text, attach_start_game_seat_transition,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::SocialUiState;
use bevy::prelude::*;
use leocard_doudizhu::{Action, Phase};
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    table: Entity,
    game: &DoudizhuSnapshot,
    avatars: &AvatarImages,
    social: &SocialUiState,
    assets: &UiAssets,
    intro: bool,
) {
    let n = game.rules.mode.players();
    let own_seat = game
        .players
        .iter()
        .find(|p| p.id == game.you)
        .map_or(0, |p| p.seat.0);
    for player in &game.players {
        let relative = (player.seat.0 + n - own_seat) % n;
        let mut node = Node {
            position_type: PositionType::Absolute,
            width: px(150),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        };
        match relative {
            0 => {
                node.left = px(18);
                node.bottom = px(4);
            }
            1 => {
                node.left = px(34);
                node.top = percent(22);
            }
            2 if n == 4 => {
                node.left = percent(43);
                node.top = px(4);
            }
            _ => {
                node.right = px(34);
                node.top = percent(22);
            }
        }
        let slot = spawn_node(commands, table, node, None);
        let active = game.current == player.id && game.result.is_none();
        let portrait = add_player_portrait(
            commands,
            slot,
            Node {
                width: px(112),
                height: px(94),
                ..default()
            },
            PlayerPortraitSpec {
                player: player.id,
                profile: PlayerMenuProfile {
                    name: &player.name,
                    avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: &player.game_profiles,
                },
                avatar_size: 58.0,
                auto_play: player.auto_play,
                menu_open: social.interaction_menu_open == Some(player.id),
                name_color: if !player.connected {
                    MUTED
                } else if active {
                    ACCENT
                } else {
                    TEXT
                },
            },
            assets,
        );
        attach_start_game_seat_transition(commands, portrait.portrait, player.id, intro);
        let role = if game.phase == Phase::Bidding {
            "待定"
        } else if game.landlord == Some(player.id) {
            "地主"
        } else {
            "农民"
        };
        add_text(
            commands,
            slot,
            format!("{role}  {} 张", player.hand_len),
            15.0,
            if active { ACCENT } else { TEXT },
            assets,
        );
        if game.phase == Phase::Bidding {
            if let Some(bid) = game.bids[usize::from(player.id.0)] {
                add_text(
                    commands,
                    slot,
                    if bid == 0 {
                        "不叫".to_owned()
                    } else {
                        format!("{bid} 分")
                    },
                    17.0,
                    ACCENT,
                    assets,
                );
            }
        } else if matches!(
            game.last_actions[usize::from(player.id.0)],
            Some(Action::Pass)
        ) {
            add_text(commands, slot, "不出", 17.0, MUTED, assets);
        }
        if let Some(hand) = game.revealed_hands.get(usize::from(player.id.0)) {
            card_row(commands, slot, hand, CardSize::FinishedHand, assets);
        }
    }
}
