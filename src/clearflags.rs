use super::*;
use bitflags::bitflags;

bitflags! {
    /// Bitflags that indicate which buffer is to be cleared.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ClearFlags: u32 {
        const COLOR = COLOR_BUFFER_BIT;
        const DEPTH = DEPTH_BUFFER_BIT;
        const STENCIL = STENCIL_BUFFER_BIT;
    }
}
