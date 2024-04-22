use core::cell::RefCell;
use critical_section::Mutex;
use embedded_graphics::mono_font::ascii::FONT_7X14;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::Text;
use esp_hal::gpio::{self, Input, PullDown};
use log::info;

use crate::ST7735;
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb565};
use embedded_hal::{digital::OutputPin, spi};
use esp_hal::delay::Delay;
use esp_hal::{
    peripherals::TIMG0,
    prelude::*,
    timer::{Timer, Timer0},
};
use st7735_lcd::ST7735;

use crate::time::{UpdateIndex, NOW};

// 时钟中断标志
pub static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>>>> =
    Mutex::new(RefCell::new(None));

// 字体与颜色
const TEXT_COLOR: Rgb565 = Rgb565::WHITE;
pub const BG_COLOR: Rgb565 = Rgb565::BLACK;
static STYLE: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_7X14, TEXT_COLOR);
static NUM: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

static mut TIME_TEXT: (
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
) = (
    (
        Text::new(NUM[0], Point::new(40, 40), STYLE),
        Text::new(NUM[0], Point::new(50, 40), STYLE),
    ),
    (
        Text::new(NUM[0], Point::new(60, 40), STYLE),
        Text::new(NUM[0], Point::new(70, 40), STYLE),
    ),
    (
        Text::new(NUM[0], Point::new(80, 40), STYLE),
        Text::new(NUM[0], Point::new(90, 40), STYLE),
    ),
);

pub static UP_BUTTON: Mutex<RefCell<Option<gpio::Gpio8<Input<PullDown>>>>> =
    Mutex::new(RefCell::new(None));
pub static DOWN_BUTTON: Mutex<RefCell<Option<gpio::Gpio9<Input<PullDown>>>>> =
    Mutex::new(RefCell::new(None));

#[handler]
#[ram]
pub fn io_interrupt() {
    esp_println::println!("GPIO Interrupt");

    critical_section::with(|cs| {
        let mut binding = (UP_BUTTON.borrow_ref_mut(cs), DOWN_BUTTON.borrow_ref_mut(cs));
        let button_up = binding.0.as_mut().unwrap();
        let button_down = binding.1.as_mut().unwrap();
        info!("button_up {}", button_up.is_low());
        info!("button_down {}", button_down.is_low());

        unsafe {
            log::info!("按钮中断：{}", NOW);
            for index in match button_up.is_low() {
                true => NOW.add_sec(),
                false => NOW.sub_sec(),
            }
            .iter()
            {
                match &index {
                    UpdateIndex::Hour10 => {
                        TIME_TEXT
                            .0
                             .0
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.0 .0.text = NUM[NOW.hour.0 as usize];
                        TIME_TEXT.0 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                    UpdateIndex::Hour1 => {
                        TIME_TEXT
                            .0
                             .1
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.0 .1.text = NUM[NOW.hour.1 as usize];
                        TIME_TEXT.0 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                    UpdateIndex::Min10 => {
                        TIME_TEXT
                            .1
                             .0
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.1 .0.text = NUM[NOW.min.0 as usize];
                        TIME_TEXT.1 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                    UpdateIndex::Min1 => {
                        TIME_TEXT
                            .1
                             .1
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.1 .1.text = NUM[NOW.min.1 as usize];
                        TIME_TEXT.1 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                    UpdateIndex::Sec10 => {
                        TIME_TEXT
                            .2
                             .0
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.2 .0.text = NUM[NOW.sec.0 as usize];
                        TIME_TEXT.2 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                    UpdateIndex::Sec1 => {
                        TIME_TEXT
                            .2
                             .1
                            .bounding_box()
                            .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                            .draw(&mut *ST7735.as_mut_ptr())
                            .unwrap();
                        TIME_TEXT.2 .1.text = NUM[NOW.sec.1 as usize];
                        TIME_TEXT.2 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                    }
                }
            }
        }

        button_up.clear_interrupt();
        button_down.clear_interrupt()
    });
}

// 时钟中断函数
#[handler]
pub fn tg0_t0_level() {
    unsafe {
        log::info!("时钟中断：{}", NOW);
        for index in NOW.add_sec().iter() {
            match &index {
                UpdateIndex::Hour10 => {
                    TIME_TEXT
                        .0
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.0 .0.text = NUM[NOW.hour.0 as usize];
                    TIME_TEXT.0 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Hour1 => {
                    TIME_TEXT
                        .0
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.0 .1.text = NUM[NOW.hour.1 as usize];
                    TIME_TEXT.0 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Min10 => {
                    TIME_TEXT
                        .1
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.1 .0.text = NUM[NOW.min.0 as usize];
                    TIME_TEXT.1 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Min1 => {
                    TIME_TEXT
                        .1
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.1 .1.text = NUM[NOW.min.1 as usize];
                    TIME_TEXT.1 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Sec10 => {
                    TIME_TEXT
                        .2
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.2 .0.text = NUM[NOW.sec.0 as usize];
                    TIME_TEXT.2 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Sec1 => {
                    TIME_TEXT
                        .2
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.2 .1.text = NUM[NOW.sec.1 as usize];
                    TIME_TEXT.2 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
            }
        }
    }

    //清除中断位
    critical_section::with(|cs| {
        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();
        timer0.clear_interrupt();
        timer0.start(1000u64.millis());
    });
}

pub fn 屏幕初始化<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>, delay: &mut Delay)
where
    SPI: spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    device.init(delay).unwrap();
    device.clear(BG_COLOR).unwrap();
    device
        .set_orientation(&st7735_lcd::Orientation::LandscapeSwapped)
        .unwrap(); //表示屏幕竖着来（反向）
    device.set_offset(1, 26);
}

pub fn 绘制边框<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>)
where
    SPI: spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    let (right_x, bottom_y) = (159, 79);

    for x in 0..=right_x {
        // 屏幕下边线
        device.set_pixel(x, bottom_y, 0xffff).unwrap();
    }

    for y in 0..=bottom_y {
        // 屏幕右边线
        device.set_pixel(right_x, y, 0xffff).unwrap();
    }

    for x in 0..=right_x {
        // 屏幕上边线
        device.set_pixel(x, 0, 0xffff).unwrap();
    }

    for y in 0..=bottom_y {
        // 屏幕左边线
        device.set_pixel(0, y, 0xffff).unwrap();
    }
}

pub unsafe fn 绘制数字<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>)
where
    SPI: spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    TIME_TEXT.0 .0.text = NUM[NOW.hour.0 as usize];
    TIME_TEXT.0 .0.draw(device).unwrap();

    TIME_TEXT.0 .1.text = NUM[NOW.hour.1 as usize];
    TIME_TEXT.0 .1.draw(device).unwrap();

    TIME_TEXT.1 .0.text = NUM[NOW.min.0 as usize];
    TIME_TEXT.1 .0.draw(device).unwrap();

    TIME_TEXT.1 .1.text = NUM[NOW.min.1 as usize];
    TIME_TEXT.1 .1.draw(device).unwrap();

    TIME_TEXT.2 .0.text = NUM[NOW.sec.0 as usize];
    TIME_TEXT.2 .0.draw(device).unwrap();

    TIME_TEXT.2 .1.text = NUM[NOW.sec.1 as usize];
    TIME_TEXT.2 .1.draw(device).unwrap();
}
