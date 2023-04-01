mod region;
mod system_code;

pub use region::*;
pub use system_code::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TitleId([u8; 4]);

impl TitleId {
    pub fn system_code(&self) -> SystemCode {
        self.0[0].into()
    }

    pub fn game_code(&self) -> &[u8] {
        &self.0[1..3]
    }

    pub fn region_code(&self) -> RegionCode {
        self.0[3].into()
    }
}

impl From<[u8; 4]> for TitleId {
    fn from(value: [u8; 4]) -> Self {
        TitleId(value)
    }
}

impl From<(SystemCode, [u8; 2], RegionCode)> for TitleId {
    fn from((system, game, region): (SystemCode, [u8; 2], RegionCode)) -> Self {
        TitleId([system.into(), game[0], game[1], region.into()])
    }
}

impl From<(SystemCode, &[u8; 2], RegionCode)> for TitleId {
    fn from((system, game, region): (SystemCode, &[u8; 2], RegionCode)) -> Self {
        TitleId([system.into(), game[0], game[1], region.into()])
    }
}

impl From<TitleId> for [u8; 4] {
    fn from(value: TitleId) -> Self {
        value.0
    }
}

impl From<TitleId> for SystemCode {
    fn from(value: TitleId) -> Self {
        value.system_code()
    }
}

impl From<TitleId> for RegionCode {
    fn from(value: TitleId) -> Self {
        value.region_code()
    }
}
