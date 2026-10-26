use super::{DoudizhuUiAction, DoudizhuUiState};
use crate::app::presentation::{
    ACCENT, GameButtonSpec, GameButtonTone, MUTED, TEXT, add_text, add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{UiAction, add_cozy_button, add_cozy_panel};
use bevy::prelude::*;
use leocard_doudizhu::{Action, Phase, classify, hint};
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DoudizhuSnapshot,
    ui: &DoudizhuUiState,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            height: px(54),
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    if game.result.is_some() {
        return;
    }
    if game.current != game.you {
        add_text(commands, row, "等待其他玩家行动", 16.0, MUTED, assets);
        return;
    }
    let mut button = |label: &str, action: Option<DoudizhuUiAction>, tone| {
        let mut spec = GameButtonSpec::action(assets, label, action.map(UiAction::Doudizhu), tone);
        spec.width = 100.0;
        spec.height = 44.0;
        add_textured_game_button(commands, row, assets, spec);
    };
    if game.phase == Phase::Bidding {
        button(
            "不叫",
            Some(DoudizhuUiAction::Act(Action::Bid(0))),
            GameButtonTone::Pass,
        );
        for score in 1..=3 {
            button(
                &format!("{score} 分"),
                (score > game.highest_bid).then_some(DoudizhuUiAction::Act(Action::Bid(score))),
                GameButtonTone::Play,
            );
        }
    } else {
        let cards: Vec<_> = ui.selected.iter().copied().collect();
        let selected = classify(&cards, game.rules.mode);
        let legal = selected
            .as_ref()
            .is_some_and(|p| game.table.as_ref().is_none_or(|(_, old)| p.beats(old)));
        let can_hint = hint(
            &game.your_hand,
            game.table.as_ref().map(|(_, p)| p),
            game.rules.mode,
        )
        .is_some();
        button(
            "不出",
            game.table
                .is_some()
                .then_some(DoudizhuUiAction::Act(Action::Pass)),
            GameButtonTone::Pass,
        );
        button(
            "提示",
            can_hint.then_some(DoudizhuUiAction::Hint),
            GameButtonTone::Hint,
        );
        button(
            "重选",
            (!ui.selected.is_empty()).then_some(DoudizhuUiAction::Clear),
            GameButtonTone::Hint,
        );
        button(
            "出牌",
            legal.then_some(DoudizhuUiAction::Play),
            GameButtonTone::Play,
        );
        let text = if !can_hint {
            "没有可压过的牌"
        } else {
            selected.as_ref().map_or("请选择手牌", |p| p.kind.label())
        };
        add_text(
            commands,
            row,
            text,
            14.0,
            if legal { ACCENT } else { MUTED },
            assets,
        );
    }
}

pub(super) fn guide(commands: &mut UiComposer, parent: Entity, assets: &UiAssets) {
    let panel = add_cozy_panel(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            right: px(28),
            top: px(30),
            width: px(470),
            max_height: percent(90),
            overflow: Overflow::scroll_y(),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert(GlobalZIndex(1900));
    add_text(commands, panel, "斗地主玩法", 25.0, ACCENT, assets);
    for text in [
        "三人一副牌17张加3张底牌，四人两副牌25张加8张底牌。最高叫分者拿底牌，先出牌。无人叫分重新洗牌。",
        "点数从小到大为3至A、2、小王、大王。花色不比较大小。",
        "支持单张、对子、三张、三带一、三带二、顺子、连对、飞机及带单带对、四带二和四带两对。",
        "顺子至少5张，连对至少3对，飞机至少2个三张。连续牌型不能包含2或王；飞机翅膀不能复用主体点数，带对须不同点数。",
        "四张及以上同点数为炸弹，张数多的优先；三人两王为王炸，四人必须四王。炸弹和王炸各翻倍。",
        "地主先出完则地主获胜，任何农民先出完则全体农民获胜。农民未出过牌或地主只出过一手触发春天，再翻倍。",
        "每名农民输赢为叫分乘倍数，地主输赢为全体农民之和。牌局积分与钱包金币分别记录。",
        "点击手牌选择，按住横向滑动可批量选牌；从已选牌开始滑动可批量取消，松开后确认选区。提示可选合法组合。",
        "叫分和出牌均不限时，断线由房主接管；头像菜单可互动，聊天面板可开关托管。",
    ] {
        add_text(commands, panel, text, 15.0, TEXT, assets);
    }
    add_cozy_button(
        commands,
        panel,
        "关闭",
        UiAction::Doudizhu(DoudizhuUiAction::ToggleGuide),
        assets,
        percent(100),
        42.0,
    );
}
