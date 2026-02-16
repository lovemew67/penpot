pub const DEBUG_VISIBLE: u32 = 0x01;
pub const PROFILE_REBUILD_TILES: u32 = 0x02;
pub const FAST_MODE: u32 = 0x04;
/// Settling mode: transitional state between fast mode and full quality.
/// Renders blur at reduced quality for a fast first pass after pan/zoom ends,
/// then a full-quality re-render is scheduled automatically.
pub const SETTLING_MODE: u32 = 0x08;
