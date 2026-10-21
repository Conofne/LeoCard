use super::state::SakuraState;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum SakuraScroll {
    Tunnels,
    Form,
    Nodes,
}

pub(super) fn scroll_sakura_lists(
    mut wheels: MessageReader<MouseWheel>,
    state: Res<SakuraState>,
    mut scrolls: Query<
        (
            &SakuraScroll,
            &RelativeCursorPosition,
            &mut ScrollPosition,
            &ComputedNode,
        ),
        With<SakuraScroll>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 40.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if !state.open || delta == 0.0 {
        return;
    }
    let hovered = scrolls
        .iter()
        .filter(|(_, cursor, _, _)| cursor.cursor_over())
        .map(|(kind, _, _, _)| *kind)
        .find(|kind| *kind == SakuraScroll::Nodes)
        .or_else(|| {
            scrolls
                .iter()
                .filter(|(_, cursor, _, _)| cursor.cursor_over())
                .map(|(kind, _, _, _)| *kind)
                .next()
        });
    for (kind, _, mut position, node) in &mut scrolls {
        if Some(*kind) == hovered {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}

pub(super) fn remember_sakura_scroll(
    scrolls: Query<(&SakuraScroll, &ScrollPosition)>,
    mut state: ResMut<SakuraState>,
) {
    for (kind, position) in &scrolls {
        match kind {
            SakuraScroll::Tunnels => state.tunnel_scroll_y = position.y,
            SakuraScroll::Form => state.form_scroll_y = position.y,
            SakuraScroll::Nodes => state.node_scroll_y = position.y,
        }
    }
}
