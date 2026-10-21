use super::{AchievementCategory, AchievementDefinition, personal};
use crate::{AchievementTrigger, TrophyCounts};
use std::collections::HashSet;
use std::ops::Deref;

type Derivation = fn(&AchievementTrigger) -> Vec<AchievementTrigger>;

/// Explicitly assembled once, then shared immutably by progress and presentation.
/// The framework owns personal achievements; games register their own rules.
pub struct AchievementRegistry {
    definitions: Vec<AchievementDefinition>,
    derivations: Vec<Derivation>,
}

impl Default for AchievementRegistry {
    fn default() -> Self {
        let mut registry = Self {
            definitions: Vec::new(),
            derivations: Vec::new(),
        };
        registry.register(personal::DEFINITIONS);
        registry
    }
}

impl Deref for AchievementRegistry {
    type Target = [AchievementDefinition];

    fn deref(&self) -> &Self::Target {
        &self.definitions
    }
}

impl AchievementRegistry {
    /// Validate programmer-authored definitions before publishing the registry.
    /// A duplicate or invalid criterion is a construction error, never a save error.
    pub fn register(&mut self, definitions: &'static [AchievementDefinition]) {
        let mut ids: HashSet<_> = self
            .definitions
            .iter()
            .map(|definition| definition.id)
            .collect();
        for definition in definitions {
            assert!(
                ids.insert(definition.id),
                "duplicate achievement: {}",
                definition.id
            );
            let criteria: HashSet<_> = definition
                .criteria
                .iter()
                .map(|criterion| criterion.id)
                .collect();
            assert_eq!(
                criteria.len(),
                definition.criteria.len(),
                "duplicate criterion: {}",
                definition.id
            );
            assert!(
                !definition.requirements.is_empty(),
                "missing requirements: {}",
                definition.id
            );
            assert!(
                definition
                    .criteria
                    .iter()
                    .all(|criterion| criterion.target > 0),
                "zero target: {}",
                definition.id
            );
            assert!(
                definition
                    .requirements
                    .iter()
                    .all(|group| !group.is_empty() && group.iter().all(|id| criteria.contains(id))),
                "unknown or empty requirement: {}",
                definition.id
            );
        }
        self.definitions.extend_from_slice(definitions);
    }

    pub fn register_derivation(&mut self, derive: Derivation) {
        self.derivations.push(derive);
    }

    pub fn by_id(&self, id: &str) -> Option<&AchievementDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.id == id)
    }

    pub fn in_category(
        &self,
        category: AchievementCategory,
    ) -> impl Iterator<Item = &AchievementDefinition> {
        self.definitions
            .iter()
            .filter(move |definition| definition.category == category)
    }

    pub fn counts<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> TrophyCounts {
        let mut seen = HashSet::new();
        let mut counts = [0_u32; 3];
        for id in ids {
            if seen.insert(id)
                && let Some(definition) = self.by_id(id)
            {
                counts[definition.tier.medal_index()] += 1;
            }
        }
        TrophyCounts {
            gold: counts[0],
            silver: counts[1],
            bronze: counts[2],
        }
    }

    pub(crate) fn derive(&self, event: &AchievementTrigger) -> Vec<AchievementTrigger> {
        self.derivations
            .iter()
            .flat_map(|derive| derive(event))
            .collect()
    }
}
