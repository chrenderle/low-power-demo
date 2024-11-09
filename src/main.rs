#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::{Spawner};
use embassy_stm32::exti::{ExtiInput, Channel};
use embassy_stm32::gpio::{Input, Pull, AnyPin, Level, Speed, Output, Pin};
#[cfg(feature = "low-power")]
use embassy_stm32::low_power::Executor;
use embassy_stm32::rcc::{LsConfig, MSIRange};
use embassy_time::Timer;
use embassy_stm32::rtc::{self, Rtc, RtcConfig};
use futures::pending;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[cfg(not(feature = "low-power"))]
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    use embassy_stm32::rcc::mux::Lptimsel;

    info!("hello1");
    let mut config = embassy_stm32::Config::default();
    config.rcc.ls = LsConfig::default_lse();
    config.rcc.mux.lptim1sel = Lptimsel::MSIK;
    config.rcc.msik = Some(MSIRange::RANGE_4MHZ);
    let p = embassy_stm32::init(config);
    info!("hello2");
    
    spawner.spawn(blinky(p.PC7.into())).unwrap();
    
    //let button = ExtiInput::new(Input::new(p.PC13.degrade(), Pull::Down), p.EXTI13.degrade());
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down);
    spawner.spawn(button_wait(button, p.PG2.into())).unwrap();
    
    //spawner.spawn(lptim(p.LPTIM1)).unwrap();    
}

#[cfg(feature = "low-power")]
#[cortex_m_rt::entry]
fn main() -> ! {
    info!("before executor");
    Executor::take().run(|spawner| {
        info!("hello1");
        let mut config = embassy_stm32::Config::default();
        config.rcc.ls = LsConfig::default_lsi();
        if cfg!(feature = "debug-during-sleep") {
            config.enable_debug_during_sleep = true;
        } else {
            config.enable_debug_during_sleep = false;
        }
        let p = embassy_stm32::init(config);
        let rtc = Rtc::new(p.RTC, RtcConfig::default());
        static RTC: StaticCell<Rtc> = StaticCell::new();
        let rtc = RTC.init(rtc);
        embassy_stm32::low_power::stop_with_rtc(rtc);
        info!("hello6");

        //spawner.spawn(blinky(p.PC7.into())).unwrap();
        
        let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down);
        spawner.spawn(button_wait(button, p.PG2.into())).unwrap();
        
        spawner.spawn(timeout()).unwrap();
    })
}

#[embassy_executor::task]
async fn timeout() -> ! {
    Timer::after_secs(60).await;
    
    loop {}
}

#[embassy_executor::task]
async fn blinky(led: AnyPin) -> ! {
    info!("hello7");
    let mut led = Output::new(led, Level::Low, Speed::Low);
    let mut counter = 0;
    loop {
        led.toggle();
        info!("toggle led: {}", counter);
        counter += 1;
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn button_wait(mut button: ExtiInput<'static>, led: AnyPin) -> ! {
    let mut led = Output::new(led, Level::Low, Speed::Low);
    loop {
        let lptim = embassy_stm32::pac::LPTIM1;
        let rcc = embassy_stm32::pac::RCC;
        info!("cfgr: {}", lptim.cfgr().read().0);
        info!("cr: {}", lptim.cr().read().0);
        info!("cnt: {}", lptim.cnt().read().0);
        info!("arr: {}", lptim.arr().read().0);
        info!("ccr: {}", lptim.ccr(0).read().0);
        info!("icr: {}", lptim.icr().read().0);
        info!("isr: {}", lptim.isr().read().0);
        info!("dier: {}", lptim.dier().read().0);
        info!("apb3enr: {}", rcc.apb3enr().read().0);
        info!("ccipr3: {}", rcc.ccipr3().read().0);
        info!("waiting for button");        
        button.wait_for_rising_edge().await;
        led.toggle();
        //Timer::after_millis(200).await;
    }
}

#[embassy_executor::task]
async fn lptim(lptim: embassy_stm32::peripherals::LPTIM1) -> ! {
    let timer = embassy_stm32::lptim::timer::Timer::new(lptim);
    timer.enable();
    embassy_stm32::pac::LPTIM1.arr().write(|w| w.set_arr(100));
    timer.continuous_mode_start();
    loop {
        pending!();
    }
}