use crate::{
    AchievementCounts, ChatContent, ChatMessage, PlayerInteraction, PlayerInteractionKind,
};
use leocard_achievements::{
    ChatActivity, ChatFacts, InteractionFacts, InteractionKind, ParticipantId, TrophyCounts,
};

impl From<&PlayerInteraction> for InteractionFacts {
    fn from(event: &PlayerInteraction) -> Self {
        Self {
            source: ParticipantId(event.source.0),
            target: ParticipantId(event.target.0),
            elapsed_millis: event.elapsed_millis,
            kind: match event.kind {
                PlayerInteractionKind::Flower => InteractionKind::Flower,
                PlayerInteractionKind::Egg => InteractionKind::Egg,
                PlayerInteractionKind::Wine => InteractionKind::Wine,
                PlayerInteractionKind::Shoe => InteractionKind::Shoe,
            },
        }
    }
}

impl From<&ChatMessage> for ChatFacts {
    fn from(message: &ChatMessage) -> Self {
        Self {
            source: ParticipantId(message.source.0),
            content: match &message.content {
                ChatContent::Text(text) => ChatActivity::Text(text.clone()),
                ChatContent::QuickVoice(_) => ChatActivity::QuickVoice,
                ChatContent::Emoji(_) => ChatActivity::Other,
            },
        }
    }
}

impl From<TrophyCounts> for AchievementCounts {
    fn from(counts: TrophyCounts) -> Self {
        Self {
            gold: counts.gold,
            silver: counts.silver,
            bronze: counts.bronze,
        }
    }
}
