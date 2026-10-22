use super::support::{DEAL_INTERVAL, started_session};
use leocard_shengji::{
    GameState, Phase, ShengjiCard, ShengjiPlayerId, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    TeamProgress, build_deck,
};

#[test]
fn dealer_partner_bot_can_bid_when_someone_else_receives_the_final_card() {
    let (mut session, _, _) = started_session();
    let rules = ShengjiRuleSet::default();
    let mut deck = build_deck();
    // Give the partner two heart level cards early, but it must wait for all four hands.
    for (position, copy) in [(2, 0), (6, 1)] {
        let index = deck
            .iter()
            .position(|card| {
                *card == ShengjiCard::suited(copy, ShengjiSuit::Heart, ShengjiRank::Two)
            })
            .unwrap();
        deck.swap(position, index);
    }
    session.game = Some(
        GameState::new(
            rules,
            TeamProgress::default(),
            Some(ShengjiPlayerId(0)),
            ShengjiPlayerId(0),
            deck,
        )
        .unwrap(),
    );
    session.room.players[2].is_bot = true;
    session.advance_time(DEAL_INTERVAL * 99);
    assert!(session.game.as_ref().unwrap().bidding().current().is_none());
    session.advance_time(DEAL_INTERVAL);
    let game = session.game.as_ref().unwrap();
    assert_eq!(game.phase(), &Phase::BiddingGrace);
    assert_eq!(game.bidding().current().unwrap().player, ShengjiPlayerId(2));
}
