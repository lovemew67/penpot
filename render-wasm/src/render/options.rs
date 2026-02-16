use crate::options;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct RenderOptions {
    pub flags: u32,
    pub dpr: Option<f32>,
}

impl RenderOptions {
    pub fn is_debug_visible(&self) -> bool {
        self.flags & options::DEBUG_VISIBLE == options::DEBUG_VISIBLE
    }

    pub fn is_profile_rebuild_tiles(&self) -> bool {
        self.flags & options::PROFILE_REBUILD_TILES == options::PROFILE_REBUILD_TILES
    }

    /// Use fast mode to enable / disable expensive operations
    pub fn is_fast_mode(&self) -> bool {
        self.flags & options::FAST_MODE == options::FAST_MODE
    }

    pub fn set_fast_mode(&mut self, enabled: bool) {
        if enabled {
            self.flags |= options::FAST_MODE;
        } else {
            self.flags &= !options::FAST_MODE;
        }
    }

    /// Settling mode: transitional reduced-quality render after pan/zoom ends.
    /// Blur is kept but rendered at reduced resolution/sigma so the first
    /// post-interaction frame appears quickly. A follow-up full-quality render
    /// is then scheduled.
    pub fn is_settling_mode(&self) -> bool {
        self.flags & options::SETTLING_MODE == options::SETTLING_MODE
    }

    pub fn set_settling_mode(&mut self, enabled: bool) {
        if enabled {
            self.flags |= options::SETTLING_MODE;
        } else {
            self.flags &= !options::SETTLING_MODE;
        }
    }

    /// Returns true if blur quality should be reduced (either fast mode or settling mode).
    pub fn is_reduced_quality(&self) -> bool {
        self.is_fast_mode() || self.is_settling_mode()
    }

    pub fn dpr(&self) -> f32 {
        self.dpr.unwrap_or(1.0)
    }
}
