// Render options flags
const DEBUG_VISIBLE: u32 = 0x01;
const PROFILE_REBUILD_TILES: u32 = 0x02;
const TEXT_EDITOR_V3: u32 = 0x04;
const SHOW_WASM_INFO: u32 = 0x08;

// Render performance options
// This is the extra area used for tile rendering (tiles beyond viewport).
// Higher values pre-render more tiles, reducing empty squares during pan but using more memory.
// Kept in *tile* units (not scaled by DPR): world tile count already matches
// DPR=1 (`BASE/zoom`), and interactive LOD keeps raster tiles at 512 px during
// zoom refill. Scaling interest by DPR would only inflate the cache surface.
const VIEWPORT_INTEREST_AREA_THRESHOLD: i32 = 1;
const MAX_BLOCKING_TIME_MS: i32 = 32;
const NODE_BATCH_THRESHOLD: i32 = 3;
const BLUR_DOWNSCALE_THRESHOLD: f32 = 8.0;
const ANTIALIAS_THRESHOLD: f32 = 7.0;

/// Raster resolution for tile textures relative to the view DPR.
///
/// Matching tile *count* at HiDPI still costs ~4× fill-rate when each tile is
/// `512*dpr` px. Interactive quality keeps raster tiles at 512 px (DPR=1 cost)
/// and upscales on present; Full quality uses sharp `512*dpr` tiles after settle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ContentQuality {
    Interactive,
    #[default]
    Full,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderOptions {
    pub flags: u32,
    pub dpr: f32,
    fast_mode: bool,
    /// Active while the user is interacting with a shape (drag, resize,
    /// rotate). Implies `fast_mode` semantics for expensive effects but
    /// keeps per-frame flushing enabled (unlike pan/zoom, where
    /// `render_from_cache` drives target presentation).
    interactive_transform: bool,
    /// Tile raster LOD — see [`ContentQuality`].
    content_quality: ContentQuality,
    /// Minimum on-screen size (CSS px at 1:1 zoom) above which vector antialiasing is enabled.
    pub antialias_threshold: f32,
    pub viewport_interest_area_threshold: i32,
    pub dpr_viewport_interest_area_threshold: i32,
    pub max_blocking_time_ms: i32,
    pub node_batch_threshold: i32,
    pub blur_downscale_threshold: f32,
    pub capture_frames: i32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            flags: 0,
            dpr: 1.0,
            fast_mode: false,
            interactive_transform: false,
            content_quality: ContentQuality::Full,
            antialias_threshold: ANTIALIAS_THRESHOLD,
            viewport_interest_area_threshold: VIEWPORT_INTEREST_AREA_THRESHOLD,
            dpr_viewport_interest_area_threshold: VIEWPORT_INTEREST_AREA_THRESHOLD,
            max_blocking_time_ms: MAX_BLOCKING_TIME_MS,
            node_batch_threshold: NODE_BATCH_THRESHOLD,
            blur_downscale_threshold: BLUR_DOWNSCALE_THRESHOLD,
            capture_frames: 0,
        }
    }
}

impl RenderOptions {
    pub fn is_debug_visible(&self) -> bool {
        self.flags & DEBUG_VISIBLE == DEBUG_VISIBLE
    }

    pub fn is_profile_rebuild_tiles(&self) -> bool {
        self.flags & PROFILE_REBUILD_TILES == PROFILE_REBUILD_TILES
    }

    /// Use fast mode to enable / disable expensive operations
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode
    }

    pub fn set_fast_mode(&mut self, enabled: bool) {
        self.fast_mode = enabled;
    }

    pub fn content_quality(&self) -> ContentQuality {
        self.content_quality
    }

    /// Returns `true` when the quality value changed.
    pub fn set_content_quality(&mut self, quality: ContentQuality) -> bool {
        if self.content_quality != quality {
            self.content_quality = quality;
            true
        } else {
            false
        }
    }

    /// Device-pixel size used to *rasterize* each tile at the current quality.
    /// Interactive quality stays at 512 px even when view DPR is 2 so fill-rate
    /// matches DPR=1 during pan/zoom; Full quality uses `512 * dpr`.
    pub fn raster_tile_size_px(&self) -> i32 {
        match self.content_quality {
            ContentQuality::Interactive if self.dpr > 1.05 => crate::tiles::TILE_SIZE_BASE as i32,
            _ => crate::tiles::tile_size_px_i32(self.dpr),
        }
    }

    pub fn needs_full_quality_upgrade(&self) -> bool {
        self.content_quality == ContentQuality::Interactive && self.dpr > 1.05
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_capture_frames(&mut self, capture_frames: i32) {
        self.capture_frames = capture_frames;
    }

    /// Syncs `dpr_viewport_interest_area_threshold` with the configured
    /// tile-ring size. Interest is intentionally **not** multiplied by DPR:
    /// tile textures already scale with DPR, so a 1-tile ring keeps the same
    /// device-pixel margin as DPR=1.
    fn update_dpr_viewport_interest_area_threshold(&mut self) {
        self.dpr_viewport_interest_area_threshold = self.viewport_interest_area_threshold;
    }

    /// Sets the devicePixelRatio.
    pub fn set_dpr(&mut self, value: f32) -> bool {
        if value > 0.0 && self.dpr != value {
            self.dpr = value;
            self.update_dpr_viewport_interest_area_threshold();
            return true;
        }
        false
    }

    /// Interactive transform is ON while the user is dragging, resizing
    /// or rotating a shape. Callers use it to keep per-frame flushing
    /// enabled and to render visible tiles in a single frame so tiles
    /// never appear sequentially or flicker during the gesture.
    pub fn is_interactive_transform(&self) -> bool {
        self.interactive_transform
    }

    pub fn set_interactive_transform(&mut self, enabled: bool) {
        self.interactive_transform = enabled;
    }

    pub fn is_text_editor_v3(&self) -> bool {
        self.flags & TEXT_EDITOR_V3 == TEXT_EDITOR_V3
    }

    pub fn show_wasm_info(&self) -> bool {
        self.flags & SHOW_WASM_INFO == SHOW_WASM_INFO
    }

    pub fn set_antialias_threshold(&mut self, value: f32) -> bool {
        if value.is_finite() && value > 0.0 {
            self.antialias_threshold = value;
            return true;
        }
        false
    }

    pub fn set_blur_downscale_threshold(&mut self, value: f32) -> bool {
        if value.is_finite() && value > 0.0 {
            self.blur_downscale_threshold = value;
            return true;
        }
        false
    }

    pub fn set_viewport_interest_area_threshold(&mut self, value: i32) -> bool {
        if value >= 0 && self.viewport_interest_area_threshold != value {
            self.viewport_interest_area_threshold = value;
            self.update_dpr_viewport_interest_area_threshold();
            return true;
        }
        false
    }

    pub fn set_node_batch_threshold(&mut self, value: i32) -> bool {
        if value > 0 {
            self.node_batch_threshold = value;
            return true;
        }
        false
    }

    pub fn set_max_blocking_time_ms(&mut self, value: i32) -> bool {
        if value > 0 {
            self.max_blocking_time_ms = value;
            return true;
        }
        false
    }
}
