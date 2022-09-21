
#[derive(Copy, Clone)]
pub struct ClipRecut {
    pub offset: u64,
    pub duration: Option<u64>,
}

impl ClipRecut {
    pub fn offset_duration(offset: u64, duration: u64) -> Self {
        Self {
            offset,
            duration: Some(duration),
        }
    }
    pub fn offset_end(offset: u64) -> Self {
        Self {
            offset,
            duration: None,
        }
    }
}
impl Default for ClipRecut {
    fn default() -> Self {
        Self {
            offset: 0,
            duration: None,
        }
    }
}
