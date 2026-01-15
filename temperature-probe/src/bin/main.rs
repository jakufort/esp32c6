#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use bme280::i2c::BME280;
use core::cell::RefCell;
use core::fmt::Write;
use critical_section::Mutex;
use defmt::{error, info};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Text, TextStyle};
use embedded_graphics::Drawable;
use embedded_hal_bus::i2c::CriticalSectionDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::{main, Blocking};
use mini_oled::prelude::{I2cInterface, Sh1106};
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const TEXT_STYLE: MonoTextStyle<BinaryColor> = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On)
    .background_color(BinaryColor::Off)
    .build();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let mut delay = Delay::new();

    let i2c = i2c_bus();

    let mut bme280 = bme280(&i2c, delay);
    let mut screen = screen(&i2c);

    info!("Configured");

    loop {
        match bme280.measure(&mut delay) {
            Ok(measurement) => {
                let mut temp: heapless::String<16> = heapless::String::new();
                write!(temp, "TEMP:     {:04.2}", measurement.temperature).unwrap();
                write_to_screen(0, 10, &mut screen, &temp);

                let mut humidity: heapless::String<16> = heapless::String::new();
                write!(humidity, "HUMIDITY: {:04.2}", measurement.humidity).unwrap();
                write_to_screen(0, 20, &mut screen, &humidity);

                let mut pressure: heapless::String<18> = heapless::String::new();
                write!(pressure, "PRESSURE: {:06.2}", measurement.pressure).unwrap();
                write_to_screen(0, 30, &mut screen, &pressure);
            }
            Err(_e) => {
                error!("Measurement failed");
                write_to_screen(0, 6, &mut screen, "Failed");
            }
        }

        screen.flush().unwrap();
        delay.delay_millis(1000);
    }
}

fn i2c_bus<'a>() -> Mutex<RefCell<I2c<'a, Blocking>>> {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("Embassy initialized!");

    let i2c_bus = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO6)
    .with_scl(peripherals.GPIO7);

    let i2c_ref_cell = RefCell::new(i2c_bus);
    let i2c_mutex = Mutex::new(i2c_ref_cell);
    info!("i2c initialized!");

    i2c_mutex
}

fn bme280<'a>(
    i2c: &'a Mutex<RefCell<I2c<'a, Blocking>>>,
    mut delay: Delay,
) -> BME280<CriticalSectionDevice<'a, I2c<'a, Blocking>>> {
    let mut bme280 = BME280::new_primary(CriticalSectionDevice::new(&i2c));

    bme280.init(&mut delay).unwrap_or_else(|err| {
        error!("init failed: {}", defmt::Debug2Format(&err));
        panic!("Couldn't init bme280!")
    });
    info!("bme280 initialized!");

    bme280
}

fn screen<'a>(
    i2c: &'a Mutex<RefCell<I2c<'a, Blocking>>>,
) -> Sh1106<I2cInterface<CriticalSectionDevice<'a, I2c<'a, Blocking>>>> {
    let i2c_interface = I2cInterface::new(CriticalSectionDevice::new(&i2c), 0x3C);

    let mut screen = Sh1106::new(i2c_interface);
    screen.init().unwrap_or_else(|err| {
        error!("init failed: {}", defmt::Debug2Format(&err));
        panic!("Couldn't init screen");
    });

    let fill = PrimitiveStyle::with_fill(BinaryColor::Off);
    Rectangle::new(Point::new(0, 0), Size::new(128, 64))
        .into_styled(fill)
        .draw(screen.get_mut_canvas())
        .unwrap();
    info!("screen initialized!");

    screen
}

fn write_to_screen<'a>(
    x: i32,
    y: i32,
    screen: &mut Sh1106<I2cInterface<CriticalSectionDevice<'a, I2c<'a, Blocking>>>>,
    text: &str,
) {
    Text::with_text_style(&text, Point::new(x, y), TEXT_STYLE, TextStyle::default())
        .draw(screen.get_mut_canvas())
        .unwrap();
}
