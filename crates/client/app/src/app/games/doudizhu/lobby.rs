use super::DoudizhuUiAction;
use crate::app::presentation::{
    EditableRuleSet, MUTED, RuleConfigRow, add_rule_config_row, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction, add_lobby_rules_heading,
};
use bevy::prelude::*;
use leocard_doudizhu::{Mode, RuleSet};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for RuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 110.0;

    fn update_action(self) -> UiAction {
        UiAction::Doudizhu(DoudizhuUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_doudizhu_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.doudizhu().expect("doudizhu lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    add_lobby_rules_heading(commands, page.rules, "斗地主配置", assets);
    let switched = RuleSet {
        mode: if rules.mode == Mode::Three {
            Mode::Four
        } else {
            Mode::Three
        },
    };
    let can_switch = page.can_configure
        && lobby.players.iter().filter(|p| p.connected).count()
            <= usize::from(switched.mode.players());
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "玩法模式",
            value: rules.mode.label().to_owned(),
            help: "三人使用一副牌，每人17张、底牌3张；四人使用两副牌，每人25张、底牌8张。一名地主对抗其余农民。",
            editable: can_switch,
            previous: can_switch.then_some(switched),
            next: can_switch.then_some(switched),
        },
        assets,
    );
    for text in [
        "叫分和出牌均不限时，支持手动托管、机器人与断线接管。",
        "依次叫1、2、3分，最高叫分者成为地主；无人叫分重新洗牌。",
        "炸弹和王炸各翻倍，春天及反春天再翻倍，地主承担全部农民的分数。",
        "四人模式同点数4至8张为炸弹，张数优先；两小王加两大王组成王炸。",
        "商店可购买150金币记牌器，统计自己手牌与公开出牌之外的剩余牌。",
    ] {
        add_text(commands, page.rules, text, 13.0, MUTED, assets);
    }
    let can_start = page.connected_count == usize::from(rules.mode.players())
        && lobby
            .players
            .iter()
            .all(|p| p.connected && p.ready && p.seat.is_some());
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            rules.mode.players(),
            can_start,
            "等待玩家中",
        ),
    );
}
