#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use bme280::i2c::BME280;
use critical_section::Mutex;
use defmt::{error, info, println};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Alignment, Text};
use embedded_hal_bus::i2c::CriticalSectionDevice;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use mini_oled::prelude::{DisplayRotation, I2cInterface, Sh1106};
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let mut delay = esp_hal::delay::Delay::new();

    let i2c_bus = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400))
    )
        .unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    info!("i2c initialized!");

    let i2c_ref_cell = RefCell::new(i2c_bus);
    let i2c_mutex = Mutex::new(i2c_ref_cell);

    let bme_i2c = CriticalSectionDevice::new(&i2c_mutex);
    let mut bme280 = BME280::new_primary(bme_i2c);

    bme280.init(&mut delay).unwrap_or_else(|err| {
        info!("init failed: {}", defmt::Debug2Format(&err));
        panic!("Couldn't init bme280!")
    });

    // let display_i2c = CriticalSectionDevice::new(&i2c_mutex);
    //
    // let i2c_interface = I2cInterface::new(display_i2c, 0x3C);
    //
    // let mut screen = Sh1106::new(i2c_interface);
    // screen.init().unwrap();
    //
    // screen.set_rotation(DisplayRotation::Rotate180).unwrap();
    //
    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .background_color(BinaryColor::Off)
    //     .build();

    info!("Started");

    loop {
        match bme280.measure(&mut delay) {
            Ok(measurement) => {
                println!("Temp: {}", measurement.temperature);
                println!("Humidity: {}", measurement.humidity);
                println!("Pressure: {}", measurement.pressure);
                // Text::with_alignment(
                //     "Got measurement",
                //     Point::new(20,20),
                //     text_style,
                //     Alignment::Center,
                // )
                //     .draw(screen.get_mut_canvas())
                //     .unwrap();
            }
            Err(e) => {
                println!("Error");
                // Text::with_alignment(
                //     "Failed",
                //     Point::new(20,30),
                //     text_style,
                //     Alignment::Center
                // )
                //     .draw(screen.get_mut_canvas())
                //     .unwrap();
            }
        }

        // screen.flush().unwrap();
        Timer::after(Duration::from_secs(1)).await;
    }
}
