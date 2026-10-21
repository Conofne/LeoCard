use super::{collection, fans, milestones};
use leocard_achievements::{AchievementDefinition, AchievementRegistry, combine};

const COUNT: usize =
    fans::DEFINITIONS.len() + milestones::DEFINITIONS.len() + collection::DEFINITIONS.len();
pub const DEFINITIONS: &[AchievementDefinition] = &combine::<COUNT>(&[
    fans::DEFINITIONS,
    milestones::DEFINITIONS,
    collection::DEFINITIONS,
]);

pub fn register(registry: &mut AchievementRegistry) {
    registry.register(DEFINITIONS);
}
