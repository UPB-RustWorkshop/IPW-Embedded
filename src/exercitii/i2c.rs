#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::info;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c as _;
use embassy_rp::peripherals::I2C0;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());


    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    const TARGET_ADDR: u8 = 0x50;
    let tx_buf1 = [0x00, 0x00, 0xf2, 0xa5, 0x15,0x27, 0x8b, 0x68, 0x6b, 0x91, 0x24, 0x32, 0x85, 0xc3, 0x25, 0xf0, 0xc6, 0x9c, 0x62, 0xfe, 0x34, 0x95, 0x35, 0x95, 0x4f, 0x61, 0x82, 0xcc, 0x07, 0xbf, 0x5f, 0xba ];

    i2c.write(TARGET_ADDR, &tx_buf1).await.unwrap();

    info!("Wrote: {:?}", tx_buf1);
    Timer::after_secs(2).await;
    let mut rx_buf = [0x00u8; 3];

    let mem_address = [0x00, 0x05];
    i2c.write_read(TARGET_ADDR, &mem_address, &mut rx_buf).await.unwrap();

    info!("Read: {:?}", rx_buf);

    loop {
        nop();
        Timer::after_secs(1).await;
    }

}