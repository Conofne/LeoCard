use crate::app::presentation::{
    ACCENT, DANGER, GameSummaryActions, GameSummaryAnimation, GameSummaryModal, GameSummaryRow,
    MUTED, PlayerMenuProfile, READY, SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL,
    SUMMARY_ROW_START_DELAY, SummaryDescriptor, TEXT, add_animated_summary_text, add_ready_avatar,
    summary_modal_visual, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{SummaryActions, add_cozy_panel, add_summary_actions};
use bevy::prelude::*;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn descriptor(game: &DoudizhuSnapshot) -> Option<SummaryDescriptor> {
    let result = game.result.as_ref()?;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: None,
        entry_count: game.players.len(),
        nonnegative_outcome: result
            .scores
            .get(usize::from(game.you.0))
            .is_none_or(|s| *s >= 0),
        reveal_duration: 0.4,
    })
}

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DoudizhuSnapshot,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
    assets: &UiAssets,
) {
    let Some(result) = &game.result else {
        return;
    };
    let panel = add_cozy_panel(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: px(15),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        GlobalZIndex(1200),
        GameSummaryModal,
        UiTransform::from_translation(Val2::px(
            0.0,
            summary_modal_visual(animation.elapsed).offset_y,
        )),
        if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    add_animated_summary_text(
        commands,
        panel,
        if result.landlord_won {
            "地主获胜"
        } else {
            "农民获胜"
        },
        27.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    add_animated_summary_text(
        commands,
        panel,
        format!(
            "底分 {}  倍数 ×{}{}",
            game.highest_bid,
            game.multiplier,
            if result.spring { "  春天" } else { "" }
        ),
        15.0,
        MUTED,
        0.0,
        animation.elapsed,
        assets,
    );
    for (index, player) in game.players.iter().enumerate() {
        let delay = SUMMARY_ROW_START_DELAY + index as f32 * SUMMARY_ROW_INTERVAL;
        let progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            panel,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            BackgroundColor(Color::NONE),
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
            if progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        add_ready_avatar(
            commands,
            row,
            player.id,
            PlayerMenuProfile {
                name: &player.name,
                avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: player.reference_points,
                completed_games: player.completed_games,
                game_profiles: &player.game_profiles,
            },
            36.0,
            player.ready,
            assets,
        );
        let role = if game.landlord == Some(player.id) {
            "地主"
        } else {
            "农民"
        };
        add_animated_summary_text(
            commands,
            row,
            format!("{}  {role}", player.name),
            16.0,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        let score = result.scores[usize::from(player.id.0)];
        add_animated_summary_text(
            commands,
            row,
            format!("{score:+} 分"),
            22.0,
            if score >= 0 { READY } else { DANGER },
            delay,
            animation.elapsed,
            assets,
        );
    }
    let actions = spawn_node(
        commands,
        panel,
        Node {
            justify_content: JustifyContent::Center,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        },
        None,
    );
    let delay = SUMMARY_ROW_START_DELAY
        + game.players.len() as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    commands.entity(actions).insert((
        GameSummaryActions { delay },
        if animation.elapsed >= delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    add_summary_actions(
        commands,
        actions,
        SummaryActions {
            is_host: game.you == game.host,
            ready: game
                .players
                .iter()
                .find(|p| p.id == game.you)
                .map(|p| p.ready),
            again: "再来一局",
        },
        assets,
    );
}
