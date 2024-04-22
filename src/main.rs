#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::{self, Delay},
    gpio::{self, Event, GpioPin, Output, PushPull, IO},
    interrupt::{self, Priority},
    peripherals::{self, Interrupt, Peripherals},
    prelude::*,
    spi::{master::Spi, FullDuplexMode},
    timer::{TimerGroup, TimerInterrupts},
};
use esp_println::println;

mod draw;
mod time;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

use embedded_hal_bus::spi::ExclusiveDevice;
static mut ST7735: MaybeUninit<
    st7735_lcd::ST7735<
        ExclusiveDevice<
            Spi<'static, peripherals::SPI2, FullDuplexMode>,
            GpioPin<Output<PushPull>, 7>,
            Delay,
        >,
        GpioPin<Output<PushPull>, 6>,
        GpioPin<Output<PushPull>, 10>,
    >,
> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = delay::Delay::new(&clocks);

    // 初始化日志
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    // 初始化时间
    // 加载时间必须要在刷新屏幕之前，屏幕刷新太耗时了
    // 时间格式1996-12-19T16:39:57-08:00
    let now: &[u8] = include_bytes!("../assets/time.bin");
    log::info!("解析时间： {:?}", now);

    use crate::time::NOW;
    unsafe {
        NOW.build(now);
        log::info!("运行时获得时间： {}", NOW);
    }

    // 屏幕外设初始化
    // SCK->2 SDA->3 RES->10 DC->6 CS->7
    io.set_interrupt_handler(io_interrupt);
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

    unsafe {
        use crate::draw::{屏幕初始化, 绘制数字, 绘制边框};
        ST7735
            .as_mut_ptr()
            .write(st7735_lcd::ST7735::new(spi, dc, res, false, true, 110, 161));
        屏幕初始化(&mut *ST7735.as_mut_ptr(), &mut delay);
        绘制边框(&mut *ST7735.as_mut_ptr());
        绘制数字(&mut *ST7735.as_mut_ptr());
    }

    println!("drew down");

    // 按钮中断初始化，我仔细检查了很多遍，esp32的引脚中断只有一个，只能手动读来区分不同的引脚了
    //dwkey->5 center->4 upkey->8 lkey->9 rkey->13
    use crate::draw::{io_interrupt, DOWN_BUTTON, UP_BUTTON};
    let mut button = io.pins.gpio8.into_pull_down_input();
    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        UP_BUTTON.borrow_ref_mut(cs).replace(button)
    });
    let mut button = io.pins.gpio9.into_pull_down_input();
    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        DOWN_BUTTON.borrow_ref_mut(cs).replace(button)
    });

    // 初始化时钟中断
    use crate::draw::tg0_t0_level;
    let timg0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        Some(TimerInterrupts {
            timer0_t0: Some(tg0_t0_level),
            ..Default::default()
        }),
    );
    let mut timer0 = timg0.timer0;

    interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority5).unwrap();
    timer0.start(1000u64.millis());
    timer0.listen();

    use crate::draw::TIMER0;
    critical_section::with(|cs| {
        TIMER0.borrow_ref_mut(cs).replace(timer0);
    });

    loop {}
}
