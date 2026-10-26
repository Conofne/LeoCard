use super::DoudizhuUiAction;
use crate::app::presentation::{ACCENT, BORDER, CardSize, UiComposer, add_card_image, spawn_node};
use crate::app::runtime::{CardFace, CardRank, CardSuit, UiAssets};
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_doudizhu::Card;
use std::collections::HashSet;

#[derive(Component)]
pub(super) struct HandCardSlot {
    pub card: Card,
    pub index: usize,
}

#[derive(Component)]
pub(super) struct HandCardSelectionOverlay {
    pub index: usize,
}

impl From<Card> for CardFace {
    fn from(card: Card) -> Self {
        let rank = match card.rank() {
            0 => CardRank::Three,
            1 => CardRank::Four,
            2 => CardRank::Five,
            3 => CardRank::Six,
            4 => CardRank::Seven,
            5 => CardRank::Eight,
            6 => CardRank::Nine,
            7 => CardRank::Ten,
            8 => CardRank::Jack,
            9 => CardRank::Queen,
            10 => CardRank::King,
            11 => CardRank::Ace,
            12 => CardRank::Two,
            13 => return Self::SmallJoker,
            _ => return Self::BigJoker,
        };
        let suit = match card.suit() {
            0 => CardSuit::Diamond,
            1 => CardSuit::Club,
            2 => CardSuit::Heart,
            _ => CardSuit::Spade,
        };
        Self::Suited { rank, suit }
    }
}

pub(super) fn card_row(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[Card],
    size: CardSize,
    assets: &UiAssets,
) {
    let column = spawn_node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    let mut sorted = cards.to_vec();
    sorted.sort_by_key(|c| (c.rank(), c.0));
    sorted.reverse();
    let per_row = match size {
        CardSize::FinishedHand => 8,
        _ => 14,
    };
    for chunk in sorted.chunks(per_row) {
        let row = spawn_node(
            commands,
            column,
            Node {
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            },
            None,
        );
        for (i, card) in chunk.iter().enumerate() {
            add_card_image(
                commands,
                row,
                *card,
                size,
                i,
                i + 1 == chunk.len(),
                false,
                assets,
            );
        }
    }
}

pub(super) fn hand(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[Card],
    selected: &HashSet<Card>,
    assets: &UiAssets,
) {
    let mut cards = cards.to_vec();
    cards.sort_by_key(|c| (c.rank(), c.0));
    cards.reverse();
    // The 33-card four-player landlord hand stays within the shared design viewport.
    let reveal = (760.0 / cards.len().max(1) as f32).clamp(18.0, 34.0);
    for (index, card) in cards.iter().enumerate() {
        let chosen = selected.contains(card);
        let slot = commands.keyed_child(
            parent,
            format!("hand:{}", card.0),
            (
                Button,
                UiAction::Doudizhu(DoudizhuUiAction::ToggleCard),
                HandCardSlot { card: *card, index },
                RelativeCursorPosition::default(),
                Node {
                    width: px(if index + 1 == cards.len() {
                        92.0
                    } else {
                        reveal
                    }),
                    height: px(142),
                    flex_shrink: 0.0,
                    position_type: PositionType::Relative,
                    ..default()
                },
            ),
        );
        let face = commands.child(
            slot,
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(92),
                    height: px(126),
                    bottom: px(if chosen { 16 } else { 0 }),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.face(*card)),
                BorderColor::all(if chosen { ACCENT } else { BORDER }),
                ZIndex(index as i32),
                Pickable::IGNORE,
            ),
        );
        commands.child(
            face,
            (
                HandCardSelectionOverlay { index },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
            ),
        );
    }
}
