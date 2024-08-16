#![no_std]
#![no_main]

use core::i32;

use embassy_embedded_hal::SetConfig;
use embassy_rp::{config, gpio::{Input, Output, Pull}, peripherals::{self, PIN_12}};
use embassy_time::Timer;
use embassy_executor::Spawner;
use embassy_usb::msos::ConfigurationSubsetHeader;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_rp::pwm::Pwm;
use embassy_futures::select::{select, select4, Either::{self, First, Second}};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;


static CHANNEL: Channel<ThreadModeRawMutex, i32, 64> = Channel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut config: ConfigPwm = Default::default();
    config.top = 0x8000;
    config.compare_a = 0;
    config.compare_b = 0;


    //66,227,250 - BLEU

    //PIN0 = RED
    //PIN1 = GREEN
    //PIN2 = BLUE

    //255,61,203 - ROZ

    let mut config1: ConfigPwm = Default::default();
    config1.top = config.top;
    config1.compare_a = config.top / 100 * 74;
    config1.compare_b = config.top / 100 * 11;

    let mut config2: ConfigPwm = Default::default();
    config2.top = config.top;
    config2.compare_a = config.top / 100 * 2;
    config2.compare_b = 0;

    let mut config1ROZ: ConfigPwm = Default::default();
    config1ROZ.top = config.top;
    config1ROZ.compare_a = 0;
    config1ROZ.compare_b = config.top / 100 * 75;

    let mut config2ROZ: ConfigPwm = Default::default();
    config2ROZ.top = config.top;
    config2ROZ.compare_a = config.top / 100 * 20;
    config2ROZ.compare_b = 0;

    //106,248,68 - VERDE DESCHIS

    let mut config1VERDE: ConfigPwm = Default::default();
    config1VERDE.top = config.top;
    config1VERDE.compare_a = config.top / 100 * 58;
    config1VERDE.compare_b = config.top / 100 * 3;

    let mut config2VERDE: ConfigPwm = Default::default();
    config2VERDE.top = config.top;
    config2VERDE.compare_a = config.top / 100 * 73;
    config2VERDE.compare_b = 0;

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
    let mut buttonX = Input::new(peripherals.PIN_14, Pull::Up);
    let mut buttonY = Input::new(peripherals.PIN_15, Pull::Up);
    

    _spawner.spawn(button_pressed(buttonA, 0)).unwrap();
    _spawner.spawn(button_pressed(buttonB, 1)).unwrap();
    _spawner.spawn(button_pressed(buttonX, 2)).unwrap();
    _spawner.spawn(button_pressed(buttonY, 3)).unwrap();


    
    loop{
        let value = CHANNEL.receive().await;
        info!("Received {}", value);
        match value {
            0 => {
                pwm.set_config(&config);
                pwm2.set_config(&config);
            },
            1 => {
                pwm.set_config(&config1);
                pwm2.set_config(&config2);
            },
            2 => {
                pwm.set_config(&config1ROZ);
                pwm2.set_config(&config2ROZ);
            },
            3 => {
                pwm.set_config(&config1VERDE);
                pwm2.set_config(&config2VERDE);
            },
            _ => break
        }
    }

}

#[embassy_executor::task(pool_size = 4)]
async fn button_pressed(mut button: Input<'static>, but: i32) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(but).await;
    }
}