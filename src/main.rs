#![no_main]
#![no_std]

use panic_rtt_target as _;
use cortex_m_rt::entry;
use rtt_target::{rtt_init_default, rprintln};

#[entry]
fn main() -> ! {
    rtt_init_default!();
    rprintln!("Hello, world!");

    loop{
        
    }
}
