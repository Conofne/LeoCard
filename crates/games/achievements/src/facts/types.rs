/// Participant identity within the caller's current session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParticipantId(pub u8);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TrophyCounts {
    pub gold: u32,
    pub silver: u32,
    pub bronze: u32,
}

impl TrophyCounts {
    pub const fn by_tier(self) -> [u32; 3] {
        [self.gold, self.silver, self.bronze]
    }
}
