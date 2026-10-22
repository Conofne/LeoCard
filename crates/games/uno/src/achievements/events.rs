use crate::UnoActionStatistics;
use leocard_achievements::{GameFacts, ParticipantId};
#[derive(Clone, Debug)]
pub enum UnoAchievementEvent {
    ActionAnalyzed {
        player: ParticipantId,
        statistics: Box<UnoActionStatistics>,
    },
}

impl From<UnoAchievementEvent> for GameFacts {
    fn from(event: UnoAchievementEvent) -> Self {
        Self::new(event)
    }
}
