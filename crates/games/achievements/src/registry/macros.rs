#[macro_export]
macro_rules! achievement_award {
    ($namespace:literal, $category:ident, $id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr, $scope:ident) => {
        $crate::AchievementDefinition {
            id: concat!($namespace, $id),
            category: $crate::AchievementCategory::$category,
            title: $title,
            tier: $crate::AchievementTier::$tier,
            description: $description,
            criteria: &[$crate::AchievementCriterion {
                id: "progress",
                amount: $amount,
                target: $target,
                scope: $crate::AchievementScope::$scope,
            }],
            requirements: &[&["progress"]],
        }
    };
}

pub use crate::achievement_award as award;
