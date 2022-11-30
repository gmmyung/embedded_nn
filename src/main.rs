#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use panic_rtt_target as _;
// use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::delay::Delay;
use stm32f3xx_hal::gpio::{Edge, Input};
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::{interrupt, prelude::*};

enum LedMode {
    Blink,
    Stay,
}

static LEDSTATE: Mutex<RefCell<Option<LedMode>>> = Mutex::new(RefCell::new(Some(LedMode::Blink)));
type ButtonPin = stm32f3xx_hal::gpio::PA0<Input>;
static BUTTON: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    let dp = Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut syscfg = dp.SYSCFG.constrain(&mut rcc.apb2);
    let mut flash = dp.FLASH.constrain();
    let mut exti = dp.EXTI;
    let clocks = rcc.cfgr.sysclk(16.MHz()).freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let mut pe9 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mut user_button = gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    syscfg.select_exti_interrupt_source(&user_button);
    user_button.trigger_on_edge(&mut exti, Edge::Rising);
    user_button.enable_interrupt(&mut exti);
    let interrupt_num = user_button.interrupt();
    cortex_m::interrupt::free(|cs| *BUTTON.borrow(cs).borrow_mut() = Some(user_button));
    unsafe { NVIC::unmask(interrupt_num) };

    loop {
        cortex_m::interrupt::free(|cs| {
            match LEDSTATE.borrow(cs).borrow().as_ref().unwrap() {
                LedMode::Blink => {
                    pe9.set_high().unwrap();
                    rprintln!("LED ON!");
                    delay.delay_ms(100.milliseconds());
                    pe9.set_low().unwrap();
                    rprintln!("LED OFF!");
                    delay.delay_ms(300.milliseconds());
                    // panic!("test panic")
                }
                LedMode::Stay => {
                    pe9.set_high().unwrap();
                }
            }
        });
    }
}

#[interrupt]
fn EXTI0() {
    rprintln!("Button Pressed!");
    cortex_m::interrupt::free(|cs| {
        // Clear the interrupt pending bit so we don't infinitely call this routine
        BUTTON
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        let mut led_ref = LEDSTATE.borrow(cs).borrow_mut();
        match led_ref.as_ref().unwrap() {
            LedMode::Blink => {
                *led_ref.as_mut().unwrap() = LedMode::Stay;
            }
            LedMode::Stay => {
                *led_ref.as_mut().unwrap() = LedMode::Blink;
            }
        }
    });
}
