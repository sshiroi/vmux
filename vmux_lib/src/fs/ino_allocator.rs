pub struct InoAllocator {
    pub latest_safe: u64,
    pub freed_inos: Vec<u64>,

    pub reuse_freed: bool,
}

impl InoAllocator {
    pub fn new() -> InoAllocator {
        InoAllocator {
            latest_safe: 5,
            freed_inos: Vec::new(),
            reuse_freed: false,
        }
    }

    pub fn allocate(&mut self) -> u64 {
        if self.reuse_freed {
            if let Some(e) = self.freed_inos.pop() {
                return e;
            }
        }
        self.latest_safe += 1;
        self.latest_safe - 1
    }

    pub fn free(&mut self, i: u64) {
        if self.reuse_freed {
            self.freed_inos.push(i);
        }
    }
}
