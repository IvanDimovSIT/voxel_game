use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
pub enum ItemId {
    Compass,
    DayClock,
    NightClock,
}
impl ItemId {
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ItemId::Compass => "Compass (return to start)",
            ItemId::DayClock => "Clock (set time to day)",
            ItemId::NightClock => "Clock (set time to night)",
        }
    }
}
