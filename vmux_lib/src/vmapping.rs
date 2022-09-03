use std::io::*;

pub struct CyclicCustomInfo {
    pub fna: Box<dyn FnMut(u64, &mut [u8]) -> bool + Send + Sync>,
    pub cycle: u64,
}

pub enum VRead {
    Static(Vec<u8>),
    Custom(Box<dyn FnMut(u64, u64, &mut dyn Write) -> u64 + Send + Sync>),
    //TODO: unneded
    CyclicCustom(CyclicCustomInfo),
}

impl VRead {
    pub fn vread(&mut self, addr: u64, size: u64, dest: &mut impl Write) -> u64 {
        match self {
            VRead::Static(e) => {
                let start = addr as usize;
                let mut end = (addr + size) as usize;
                if end > e.len() {
                    end = e.len();
                }

                dest.write(&e[start..end]).unwrap() as u64;
                return (end - start) as u64;
            }
            VRead::CyclicCustom(cci) => {
                let addr_to_much = addr % cci.cycle;
                let first_cycle = (addr - addr_to_much) / cci.cycle;
                let cycle_cnt = (size + (cci.cycle - (size % cci.cycle))) / cci.cycle;

                let mut size = size;
                let mut red = 0;
                for i in 0..cycle_cnt {
                    let c = first_cycle + i;
                    let mut b = vec![0; cci.cycle as usize];
                    let err = (cci.fna)(c, &mut b);
                    if err {
                        return red;
                    }
                    let towrt = u64::min(cci.cycle, size) as usize;
                    let offset = if i != 0 { 0 } else { addr_to_much as usize };
                    dest.write(&b[offset..towrt]).unwrap();
                    let rr = (towrt - offset) as u64;
                    size -= rr;
                    red += rr;
                    if size == 0 {
                        break;
                    }
                }
                return red;
            }
            VRead::Custom(e) => return e(addr, size, dest),
        }
    }
}

#[test]
fn test_vreaad() {
    {
        let mut a = VRead::Static(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
        let mut dst = Vec::new();
        assert_eq!(a.vread(0, 3, &mut dst), 3);
        assert_eq!(&dst, &[0, 1, 2]);

        let mut dst = Vec::new();
        assert_eq!(a.vread(1, 2, &mut dst), 2);
        assert_eq!(&dst, &[1, 2]);

        let mut dst = Vec::new();
        assert_eq!(a.vread(11, 23, &mut dst), 3);
        assert_eq!(&dst, &[11, 12, 13]);
    }
    let fna = Box::new(|a: u64, b: &mut [u8]| {
        if a >= 4 {
            return true;
        }
        for x in 0..6 as u64 {
            b[x as usize] = (a + x) as u8;
        }
        return false;
    });
    let mut a = VRead::CyclicCustom(CyclicCustomInfo { cycle: 6, fna });
    let mut dst = Vec::new();
    assert_eq!(a.vread(0, 6, &mut dst), 6);
    assert_eq!(&dst, &[0, 1, 2, 3, 4, 5]);

    let mut dst = Vec::new();
    assert_eq!(a.vread(0, 7, &mut dst), 7);
    assert_eq!(&dst, &[0, 1, 2, 3, 4, 5, 1]);

    let mut dst = Vec::new();
    assert_eq!(a.vread(3, 7, &mut dst), 7);
    assert_eq!(&dst, &[3, 4, 5, 1, 2, 3, 4]);

    let mut dst = Vec::new();
    assert_eq!(a.vread(6 * 3, 8, &mut dst), 6);
    assert_eq!(&dst, &[3, 4, 5, 6, 7, 8]);

    let mut vmp = VMap::new();
    vmp.hint_size = 4;
    vmp.add_vec(vec![1, 2, 3]);
    vmp.add_vec(vec![5, 6, 7]);
    vmp.add_vec(vec![8, 9, 10]);
    let mut dst = Vec::with_capacity(200);
    vmp.vread(1, 5, &mut dst);

    assert_eq!(dst.len(), 5);
    assert_eq!(&dst, &[2, 3, 5, 6, 7]);
}

pub struct VMapping {
    pub addr: u64,
    pub size: u64,
    pub a: Box<VRead>,
}

pub struct VMap {
    pub mappings: Vec<VMapping>,
    pub lookup_cache: Vec<usize>,
    pub addr: u64,

    pub hint_size: u64,
}

impl VMap {
    pub fn new() -> VMap {
        VMap {
            mappings: Vec::new(),
            lookup_cache: vec![],
            addr: 0,
            hint_size: 1024 * 1024 * 512,
        }
    }
    pub fn new_ex(capa: usize) -> VMap {
        VMap {
            mappings: Vec::with_capacity(capa),
            ..VMap::new()
        }
    }

    pub fn memory_footprint(&self) -> usize {
        let mappings_size = self.mappings.len() * std::mem::size_of::<VMapping>();

        let mut another = 0;
        for m in &self.mappings {
            another += match &*m.a {
                VRead::Static(e) => e.len(),
                VRead::Custom(_) => 4,
                VRead::CyclicCustom(_) => 4,
            };
        }
        another + mappings_size + self.lookup_cache.len() * std::mem::size_of::<usize>()
    }

    pub fn insert_consume_other(&mut self, other: VMap) {
        self.mappings.reserve(other.mappings.len());
        for m in other.mappings {
            self.add(m.size, m.a);
        }
    }

    pub fn total_size(&self) -> u64 {
        if self.mappings.len() == 0 {
            return 0;
        }
        let a = &self.mappings[self.mappings.len() - 1];
        let calced = a.addr + a.size;
        assert_eq!(calced, self.addr);
        calced
    }

    pub fn add(&mut self, size: u64, a: Box<VRead>) {
        self.mappings.push(VMapping {
            addr: self.addr,
            size,
            a,
        });
        self.addr += size;
    }

    pub fn add_vec(&mut self, a: Vec<u8>) {
        if a.len() == 0 {
            //Dunno what to do but 0 size vecs add bugs
            return;
        }
        self.add(a.len() as _, Box::new(VRead::Static(a)))
    }

    fn generate_lookup_cache(&mut self) {
        let hints = self.total_size() / self.hint_size;

        for h in 0..hints {
            for (i, f) in self.mappings.iter().enumerate() {
                if f.addr >= self.hint_size * h {
                    self.lookup_cache.push(if i == 0 { 0 } else { i - 1 });
                    break;
                }
            }
        }
        if self.lookup_cache.len() == 0 {
            self.lookup_cache.push(0);
        } else {
            self.lookup_cache
                .push(self.lookup_cache[self.lookup_cache.len() - 1]);
        }
    }

    pub fn vread(&mut self, addr: u64, size: u64, dest: &mut impl Write) -> u64 {
        if addr >= self.addr {
            //TODO: this should never happen but it does. investige!
            return 0;
        }

        if self.lookup_cache.len() == 0 {
            self.generate_lookup_cache();
        }

        let mut vread_addr = addr;
        let mut vread_size = size;

        let hin = self.lookup_cache[(vread_addr / self.hint_size) as usize];

        let mut bytes_red = 0;
        let mut found_start = false;

        for iii in hin..self.mappings.len() {
            let mapping = &mut self.mappings[iii];

            if mapping.addr <= vread_addr && vread_addr < mapping.addr + mapping.size {
                if !found_start {
                    found_start = true;
                }
                let offset = vread_addr - mapping.addr;
                let r_size = u64::min(vread_size, mapping.size - offset);

                let current_red = mapping.a.vread(offset, r_size, dest);
                assert_eq!(current_red, r_size);

                vread_size -= current_red;
                vread_addr += current_red;

                bytes_red += current_red;
                if vread_size == 0 {
                    return bytes_red;
                }
            } else {
                if found_start {
                    panic!("Should never happend");
                    //return bytes_red;
                }
            }
        }
        assert_eq!(found_start, true);
        bytes_red
    }
}
