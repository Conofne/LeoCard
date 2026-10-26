use super::{DoudizhuUiState, cards, controls, counter, players, summary};
use crate::app::presentation::{
    ACCENT, CardSize, GameSummaryAnimation, MUTED, StartGameSeatTransition, TableBackground,
    TableBackgroundMaterial, add_auto_play_overlay, add_text, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
use crate::app::shell::{
    ChatPanelState, SocialUiState, UiAction, add_chat_help_button, add_chat_panel,
    add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_doudizhu::Phase;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node, ui_component};

pub(crate) struct DoudizhuTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a DoudizhuSnapshot,
    pub ui: &'a DoudizhuUiState,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub summary: &'a GameSummaryAnimation,
    pub transition: &'a StartGameSeatTransition,
}

pub(crate) fn render_doudizhu_table(
    commands: &mut UiComposer,
    root: Entity,
    context: DoudizhuTable<'_>,
) {
    let game = context.game;
    let assets = context.assets;
    let content = commands.mount(root, format!("doudizhu:{:?}", game.match_id), bsn! { @ui_component(Node {
        width: percent(100), flex_grow: 1.0, min_height: px(0), position_type: PositionType::Relative,
        flex_direction: FlexDirection::Column, ..default()
    }) });
    let material = commands.material(
        content,
        context.table_materials,
        TableBackgroundMaterial {
            params: table_material_params(
                context.brightness,
                context.vignette,
                context.appearance.custom_felt.is_none(),
            ),
            texture: context
                .appearance
                .custom_felt
                .as_ref()
                .unwrap_or(&assets.table_felt)
                .clone(),
        },
    );
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let table = spawn_node(
        commands,
        content,
        Node {
            flex_grow: 1.0,
            width: percent(100),
            min_height: px(380),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let center = spawn_node(
        commands,
        table,
        Node {
            width: percent(54),
            max_width: px(640),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(12),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        center,
        game.rules.mode.label(),
        22.0,
        ACCENT,
        assets,
    );
    add_text(
        commands,
        center,
        format!(
            "底分 {}  倍数 ×{}",
            game.highest_bid.max(1),
            game.multiplier
        ),
        15.0,
        MUTED,
        assets,
    );
    if game.kitty.is_empty() {
        let backs = spawn_node(
            commands,
            center,
            Node {
                column_gap: px(3),
                ..default()
            },
            None,
        );
        for _ in 0..game.rules.mode.kitty_len() {
            commands.child(
                backs,
                (
                    Node {
                        width: px(29),
                        height: px(40),
                        ..default()
                    },
                    ImageNode::new(assets.playing_cards.card_back.clone()),
                ),
            );
        }
    } else {
        cards::card_row(
            commands,
            center,
            &game.kitty,
            crate::app::presentation::CardSize::Score,
            assets,
        );
    }
    if game.result.is_none() {
        let name = game
            .players
            .iter()
            .find(|p| p.id == game.current)
            .map_or("玩家", |p| p.name.as_str());
        let action = if game.phase == Phase::Bidding {
            "叫分"
        } else {
            "出牌"
        };
        add_text(
            commands,
            center,
            format!("{name} {action}"),
            18.0,
            ACCENT,
            assets,
        );
    }
    if let Some((player, play)) = &game.table {
        let name = game
            .players
            .iter()
            .find(|p| p.id == *player)
            .map_or("玩家", |p| p.name.as_str());
        add_text(
            commands,
            center,
            format!("{name}  {}", play.kind.label()),
            16.0,
            MUTED,
            assets,
        );
        cards::card_row(commands, center, &play.cards, CardSize::QiGuiSeat, assets);
    } else if game.phase == Phase::Playing {
        add_text(commands, center, "自由领出", 16.0, MUTED, assets);
    }
    players::render(
        commands,
        table,
        game,
        context.avatars,
        context.social,
        assets,
        context.transition.is_active_for(game.match_id),
    );
    let hand_area = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            height: px(218),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            ..default()
        },
        None,
    );
    controls::render(commands, hand_area, game, context.ui, assets);
    if !context.client.0.model().is_spectating() {
        let row = spawn_node(
            commands,
            hand_area,
            Node {
                height: px(146),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        cards::hand(commands, row, &game.your_hand, &context.ui.selected, assets);
    }
    let auto = game
        .players
        .iter()
        .find(|p| p.id == game.you)
        .map(|p| p.auto_play);
    let chat = add_chat_panel(
        commands,
        content,
        context.chat,
        assets,
        game.result.is_none().then_some(auto).flatten(),
        &[],
    );
    add_chat_help_button(
        commands,
        chat,
        UiAction::Doudizhu(super::DoudizhuUiAction::ToggleGuide),
        assets,
    );
    if auto == Some(true) && game.result.is_none() {
        add_auto_play_overlay(commands, content, assets);
    }
    counter::render(commands, content, game, context.ui, assets);
    summary::render(
        commands,
        table,
        game,
        context.avatars,
        context.summary,
        assets,
    );
    if context.ui.guide_open {
        controls::guide(commands, content, assets);
    }
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, assets);
    }
}
