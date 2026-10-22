use crate::{GuessActionStatistics, GuessHandStatistics};
use leocard_achievements::{GameFacts, ParticipantId};
#[derive(Clone, Debug)]
pub enum GuessAchievementEvent {
    ActionAnalyzed {
        player: ParticipantId,
        statistics: Box<GuessActionStatistics>,
    },
    HandAnalyzed {
        player: ParticipantId,
        statistics: GuessHandStatistics,
    },
}

impl From<GuessAchievementEvent> for GameFacts {
    fn from(event: GuessAchievementEvent) -> Self {
        Self::new(event)
    }
}
