#![no_std]
#![no_main]

use embassy_embedded_hal::SetConfig;
use embassy_futures::select;
use embassy_rp::{config, gpio::{Level, Output}, peripherals};
use embassy_time::Timer;
use embassy_executor::Spawner;
use embassy_usb::msos::ConfigurationSubsetHeader;
use fixed::const_fixed_from_int;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_rp::pwm::Pwm;
use embassy_rp::gpio::{Input, Pull};
use embassy_futures::select::{select, Either::First, Either::Second};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // let mut pin0: Output = Output::new(peripherals.PIN_0, Level::High);
    // let mut pin1: Output = Output::new(peripherals.PIN_1, Level::High);
    // let mut pin2: Output = Output::new(peripherals.PIN_2, Level::High);

    let mut config: ConfigPwm = Default::default();
    config.top = 0x8000;
    config.compare_a = config.top;
    config.compare_b = config.top;

    let procent = config.top / 10;

    let mut pwm = Pwm::new_output_ab( // output A
        peripherals.PWM_SLICE0, // channel 0
        peripherals.PIN_0, // pin 0
        peripherals.PIN_1,
        config.clone()
    );

    let mut pwm2 = Pwm::new_output_a( // output A
        peripherals.PWM_SLICE1, // channel 1
        peripherals.PIN_2, // pin 2
        config.clone()
    );

    let mut buttonA = Input::new(peripherals.PIN_12, Pull::Up);
    let mut buttonB = Input::new(peripherals.PIN_13, Pull::Up);
    
    loop{
        let select = select(buttonA.wait_for_falling_edge(), buttonB.wait_for_falling_edge()).await;
        match select{
            First(()) => {
                if config.compare_a < procent 
                    { continue; }
                config.compare_a -= procent;
                config.compare_b -= procent;
            },
            Second(()) => {
                if config.compare_a > config.top - procent
                    { continue; }
                config.compare_a += procent;
                config.compare_b += procent;
            }
        }
        pwm.set_config(&config);
        pwm2.set_config(&config);
    }

}

// #[embassy_executor::task]
// async fn increase_intensity()
// {
//     if(config.compare_a < procent)
//     {
//         return;
//     }
//     config.compare_a -= procent as u16;
// }