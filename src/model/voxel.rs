use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
pub enum Voxel {
    None,
    Stone,
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
