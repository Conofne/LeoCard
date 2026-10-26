mod bot;
mod card;
mod game;
mod play;
mod rules;

pub use bot::{automatic_action, hint};
pub use card::{Card, build_deck, rank_label};
pub use game::{Action, GameError, GameState, Phase, ResultView};
pub use play::{Play, PlayKind, classify};
pub use rules::{Mode, RuleSet};

#[cfg(test)]
mod tests;
