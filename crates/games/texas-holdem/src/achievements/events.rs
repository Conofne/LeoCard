use crate::{TexasHoldemActionStatistics, TexasHoldemHandStatistics};
use leocard_achievements::{GameFacts, ParticipantId};
#[derive(Clone, Debug)]
pub enum TexasAchievementEvent {
    HandAnalyzed {
        player: ParticipantId,
        statistics: Box<TexasHoldemHandStatistics>,
    },
    ActionApplied {
        player: ParticipantId,
        statistics: TexasHoldemActionStatistics,
    },
}

impl From<TexasAchievementEvent> for GameFacts {
    fn from(event: TexasAchievementEvent) -> Self {
        Self::new(event)
    }
}
