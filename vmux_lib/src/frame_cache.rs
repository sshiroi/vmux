pub struct FrameCache {
    pub cache: Vec<(u64, Vec<u8>)>,
    pub idx: usize,
}
impl FrameCache {
    pub fn new(frame_size: usize, cache_size: usize) -> FrameCache {
        let mut cache = Vec::with_capacity(cache_size);
        for x in 0..cache_size {
            cache.push(((u64::MAX - x as u64), Vec::with_capacity(frame_size)));
        }
        FrameCache { cache, idx: 0 }
    }

    pub fn find(&self, frame: u64) -> Option<&[u8]> {
        for f in &self.cache {
            if f.0 == frame {
                return Some(&f.1);
            }
        }

        None
    }
    pub fn contains(&self, frame: u64) -> bool {
        for f in &self.cache {
            if f.0 == frame {
                return true;
            }
        }

        false
    }

    pub fn push(&mut self, frame: u64, data: Vec<u8>) {
        assert_eq!(self.contains(frame), false);

        self.cache[self.idx] = (frame, data);
        self.idx += 1;
        if self.idx == self.cache.len() {
            self.idx = 0;
        }
    }
}
