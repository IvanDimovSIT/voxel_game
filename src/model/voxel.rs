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
impl Voxel {
    pub fn is_empty(voxel: Option<Self>) -> bool {
        if let Some(some) = voxel {
            some == Voxel::None
        } else {
            true
        }
    }
}
