#![no_std]
#![no_main]

use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut pin4: Output = Output::new(peripherals.PIN_4, Level::Low);
    let mut pin5: Output = Output::new(peripherals.PIN_5, Level::Low);
    
    loop{
        pin4.set_high();
        info!("Green led on!");
        Timer::after_millis(500).await;
        pin4.set_low();
        info!("Red led on!");
        pin5.set_high();
        Timer::after_secs(1).await;
        pin5.set_low();
    }
}
