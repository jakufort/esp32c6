#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use defmt::info;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use esp_hal::time::Rate;
use sh1106::prelude::GraphicsMode;
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut delay = esp_hal::delay::Delay::new();

    let i2c_bus = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400))
    )
        .unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    let i2c_bus = RefCell::new(i2c_bus);

    let bme_i2c = RefCellDevice::new(&i2c_bus);
    let oled_i2c = RefCellDevice::new(&i2c_bus);

    let mut bme280 = bme280::i2c::BME280::new_primary(bme_i2c);
    bme280.init(&mut delay).expect("BME280 init failed");

    let mut display = sh1106::Builder::new().connect_i2c(oled_i2c).into();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    info!("Started");

    loop {
        display.clear();
    }
}
