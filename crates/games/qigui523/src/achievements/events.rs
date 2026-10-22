use crate::QiGuiActionStatistics;
use leocard_achievements::{GameFacts, ParticipantId};
#[derive(Clone, Debug)]
pub enum QiGuiAchievementEvent {
    ActionAnalyzed {
        player: ParticipantId,
        statistics: Box<QiGuiActionStatistics>,
    },
}

impl From<QiGuiAchievementEvent> for GameFacts {
    fn from(event: QiGuiAchievementEvent) -> Self {
        Self::new(event)
    }
}
