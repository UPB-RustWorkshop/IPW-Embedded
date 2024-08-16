#![no_std]
#![no_main]

use cyw43::new;
use embassy_executor::Spawner;
use defmt::info;
use embassy_rp::peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use ipw_embedded::display::SPIDeviceInterface;
use embedded_graphics::{pixelcolor::Rgb565, text::Text};
use embedded_graphics::prelude::Point;
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::Drawable;

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
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, color);
    Text::new("Welcome to Rust Workshop!", Point::new(36, 190), style)
        .draw(&mut display)
        .unwrap();

}