#![no_std]
#![no_main]

use embassy_executor::Spawner;
use defmt::info;
//use embassy_rp::peripherals;
use embassy_time::Timer;
use rand::RngCore;
use {defmt_rtt as _, panic_probe as _};
use ipw_embedded::display::SPIDeviceInterface;
use embedded_graphics::{pixelcolor::Rgb565, text::Text};
use embedded_graphics::prelude::Point;
use embedded_graphics::mono_font::ascii:: FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::Drawable;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_rp::bind_interrupts;
use embedded_hal_async::i2c::I2c as _;
use embassy_rp::peripherals::I2C0;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_rp::clocks::RoscRng;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});


static CHANNEL: Channel<ThreadModeRawMutex, u8, 64> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    let peripherals = embassy_rp::init(Default::default());

    let miso = peripherals.PIN_4;
    let display_cs = peripherals.PIN_17;
    let mosi = peripherals.PIN_19;
    let clk = peripherals.PIN_18;
    let rst = peripherals.PIN_0;
    let dc = peripherals.PIN_16;
    let mut display_config = embassy_rp::spi::Config::default();
    display_config.frequency = 64_000_000;
    display_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    // Init SPI
    let spi: embassy_rp::spi::Spi<'_, _, embassy_rp::spi::Blocking> =
        embassy_rp::spi::Spi::new_blocking(
            peripherals.SPI0,
            clk,
            mosi,
            miso,
            display_config.clone(),
        );
    let spi_bus: embassy_sync::blocking_mutex::Mutex<
        embassy_sync::blocking_mutex::raw::NoopRawMutex,
        _,
    > = embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(spi));

    let display_spi = embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig::new(
        &spi_bus,
        embassy_rp::gpio::Output::new(display_cs, embassy_rp::gpio::Level::High),
        display_config,
    );

    let dc = embassy_rp::gpio::Output::new(dc, embassy_rp::gpio::Level::Low);
    let rst = embassy_rp::gpio::Output::new(rst, embassy_rp::gpio::Level::Low);
    let di = SPIDeviceInterface::new(display_spi, dc);

    // Init ST7789 LCD
    let mut display = st7789::ST7789::new(di, rst, 240, 240);
    display.init(&mut embassy_time::Delay).unwrap();
    display
        .set_orientation(st7789::Orientation::Portrait)
        .unwrap();
    use embedded_graphics::draw_target::DrawTarget;
    display.clear(embedded_graphics::pixelcolor::RgbColor::BLACK).unwrap();
    // ************************************************************************

    let color = Rgb565::new(255, 255, 0);
    // Write welcome message
    let style = MonoTextStyle::new(&FONT_10X20, color);
    Text::new("Prepare for the memory game!", Point::new(36, 190), style)
        .draw(&mut display)
        .unwrap();

    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());


    const TARGET_ADDR: u8 = 0x50;
    let tx_buf1 = [0x00u8; 3];
    i2c.write(TARGET_ADDR, &tx_buf1).await.unwrap();

    let mut led_green: Output = Output::new(peripherals.PIN_2, Level::Low);
    let mut led_red: Output = Output::new(peripherals.PIN_1, Level::Low);

    let buttona = Input::new(peripherals.PIN_12, Pull::Up);
    let buttonb = Input::new(peripherals.PIN_13, Pull::Up);
    let buttonx = Input::new(peripherals.PIN_14, Pull::Up);
    let buttony = Input::new(peripherals.PIN_15, Pull::Up);
    

    spawner.spawn(button_pressed(buttona, 0 as u8)).unwrap();
    spawner.spawn(button_pressed(buttonb, 1 as u8)).unwrap();
    spawner.spawn(button_pressed(buttonx, 2 as u8)).unwrap();
    spawner.spawn(button_pressed(buttony, 3 as u8)).unwrap();
    let butt_dict = ['A','B','X','Y'];

    let mut button_index = 0;
    let mut arr = [0x00u8; 4];
    let mut score = 0;
    loop {
        if button_index == 4
        {
            button_index = 0;
            score += 1;
            info!("Your score is: {}", score);
            led_green.set_high();
            Timer::after_secs(2).await;
            led_green.set_low();
        }
        if button_index == 0
        {
            arr = generate_random_sequence();
            for i in 0..4
            {
                display.clear(embedded_graphics::pixelcolor::RgbColor::BLACK).unwrap();
                if arr[i] == 0
                {
                    Text::new("A", Point::new(36, 190), style).draw(&mut display).unwrap();
                } else
                if arr[i] == 1
                {
                    Text::new("B", Point::new(36, 190), style).draw(&mut display).unwrap();
                } else
                if arr[i] == 2
                {
                    Text::new("X", Point::new(36, 190), style).draw(&mut display).unwrap();
                } else
                if arr[i] == 3
                {
                    Text::new("Y", Point::new(36, 190), style).draw(&mut display).unwrap();
                }
                Timer::after_millis(500).await;
            }
            display.clear(embedded_graphics::pixelcolor::RgbColor::BLACK).unwrap();
        }

        CHANNEL.clear();
        let value = CHANNEL.receive().await;
        info!("Pressed button {}", butt_dict[value as usize]);

        if value == arr[button_index]
        {
            button_index += 1;
            info!("Pressed correct button!");
            led_green.set_high();
            Timer::after_millis(300).await;
            led_green.set_low();
            
        }else {

            button_index = 0;
            led_red.set_high();
            info!("Pressed incorrect button!");
            display.clear(embedded_graphics::pixelcolor::RgbColor::BLACK).unwrap();
            Text::new("INCORRECT BUTTON", Point::new(36, 190), style).draw(&mut display).unwrap();
            Timer::after_secs(2).await;
            led_red.set_low();
            
            
            CHANNEL.clear();
            let mut buff = [0x00u8];
            i2c.write_read(TARGET_ADDR, &[0x00u8; 2], &mut buff).await.unwrap();
            if score > buff[0]
            {
                let mut tx_buff = [0x00u8; 3];
                tx_buff[2] = score as u8;
                i2c.write(TARGET_ADDR, &tx_buff).await.unwrap();
                info!("New highscore: {}", score);
            }

            score = 0;
        }
    }
}

#[embassy_executor::task(pool_size = 4)]
async fn button_pressed(mut button: Input<'static>, but: u8) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(but).await;
    }
}

fn generate_random_sequence()->[u8; 4]
{
    let mut arr = [0x00u8; 4];

    RoscRng.fill_bytes(&mut arr);

    for i in 0..4
    {
        arr[i] %= 4;
    }

    return arr;
}