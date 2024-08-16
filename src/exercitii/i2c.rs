#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::info;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c as _;
use embassy_rp::peripherals::I2C0;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, config};
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_rp::pwm::Pwm;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut config1: ConfigPwm = Default::default();
    config1.top = 0x8000;
    config1.compare_a = 0;
    config1.compare_b = 0;

    let mut config2: ConfigPwm = Default::default();
    config2.top = config1.top;
    config2.compare_a = 0;
    config2.compare_b = 0;

    let mut pwm = Pwm::new_output_ab( // output A
        peripherals.PWM_SLICE0, // channel 0
        peripherals.PIN_0, // pin 0
        peripherals.PIN_1,
        config1.clone()
    );

    let mut pwm2 = Pwm::new_output_a( // output A
        peripherals.PWM_SLICE1, // channel 1
        peripherals.PIN_2, // pin 2
        config2.clone()
    );

    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    const TARGET_ADDR: u8 = 0x50;
    let tx_buf1 = [0x00, 0x00, 0xf2, 0xa5, 0x15,0x27, 0x8b, 0x68, 0x6b, 0x91, 0x24, 0x32, 0x85, 0xc3, 0x25, 0xf0, 0xc6, 0x9c, 0x62, 0xfe, 0x34, 0x95, 0x35, 0x95, 0x4f, 0x61, 0x82, 0xcc, 0x07, 0xbf, 0x5f, 0xba ];

    i2c.write(TARGET_ADDR, &tx_buf1).await.unwrap();

    info!("Wrote: {:?}", tx_buf1);

    let mem_address = [0x00, 0x00];
    let mut rx_buf = [0x00u8; 3];
    
    loop {
        Timer::after_secs(2).await;
        i2c.write_read(TARGET_ADDR, &mem_address, &mut rx_buf).await.unwrap();

        info!("Read: {:?}", rx_buf);
        change_color(rx_buf, config1.clone(), config2.clone(), &mut pwm, &mut pwm2);

        for i in 0..9
        {
            Timer::after_millis(500).await;
            i2c.read(TARGET_ADDR, &mut rx_buf).await.unwrap();
            info!("Read: {:?} Color number {}", rx_buf, i+2);
            change_color(rx_buf, config1.clone(), config2.clone(), &mut pwm, &mut pwm2);
        }
    }
}

fn change_color(arr: [u8; 3], mut config1: ConfigPwm, mut config2: ConfigPwm, pwm: &mut Pwm, pwm2: &mut Pwm)
{
    let mut procent: u16 = arr[0] as u16 * 100 / 255;

    procent = 100 - procent;
    config1.compare_a = config1.top / 100 * procent as u16;

    procent = arr[1] as u16 * 100 / 255;
    procent = 100 - procent;
    config1.compare_b = config1.top / 100 * procent as u16;

    procent = arr[2] as u16 * 100 / 255;
    procent = 100 - procent;
    config2.compare_a = config2.top / 100 * procent as u16;

    pwm.set_config(&config1);
    pwm2.set_config(&config2);
}