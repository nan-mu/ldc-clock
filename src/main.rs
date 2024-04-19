#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;
// use embedded_graphics::image::Image;
// use embedded_graphics::image::ImageRaw;
// use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay,
    gpio::{self, IO},
    peripherals::Peripherals,
    prelude::*,
};
use esp_println::println;
use st7735_lcd::Orientation;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}
#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = delay::Delay::new(&clocks);

    // SCK->2 SDA->3 RES->10 DC->6 CS->7
    let sck = io.pins.gpio2;
    let sda = io.pins.gpio3;
    let res = io.pins.gpio10.into_push_pull_output();
    let dc = io.pins.gpio6.into_push_pull_output();
    let cs = io.pins.gpio7.into_push_pull_output();

    let spi = esp_hal::spi::master::Spi::new(
        peripherals.SPI2,
        100u32.kHz(),
        esp_hal::spi::SpiMode::Mode0,
        &clocks,
    )
    .with_pins(Some(sck), Some(sda), gpio::NO_PIN, gpio::NO_PIN);
    let spi = embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs, delay);
    let mut st7735 = st7735_lcd::ST7735::new(spi, dc, res, false, true, 120, 161);

    st7735.init(&mut delay).unwrap();
    st7735.clear(Rgb565::RED).unwrap();
    st7735.set_orientation(&Orientation::Portrait).unwrap();
    // st7735.set_offset(0, 20);

    // let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("../assets/ferris.raw"), 86);
    // let image = Image::new(&image_raw, Point::new(26, 8));
    // image.draw(&mut st7735).unwrap();

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");
    println!("Hello world!");
    loop {}
}
