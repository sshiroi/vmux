mod lib;

pub use lib::*;
use vmux_lib::{bd_cache::BDsCache, handling::Config};

fn main() {
    println!("Dont use this pls!");
    spawn_combined(
        Config::dflt(),
        1234,
        build_fls(&mut BDsCache::new(), &Config::dflt()),
    );
    loop {}
}
