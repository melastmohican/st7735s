#![no_std]
#![no_main]

use panic_probe as _;
use cortex_m_rt::entry;
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::Text;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics::image::Image;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal_compat::ForwardCompat;
use st7735s::{Orientation, PixelOrder, ST7735};
use stm32h7xx_hal::hal::spi;
use stm32h7xx_hal::{prelude::*, spi::NoMiso, stm32};
use tinybmp::Bmp;

struct Led<P: OutputPin> {
    pin: P,
    brightness: u8, // Duty cycle (0-10)
}

impl<P: OutputPin> Led<P> {
    pub fn new(pin: P) -> Self {
        Self {
            pin,
            brightness: 5, // Default 50% duty cycle
        }
    }

    pub fn set_brightness(&mut self, value: u8) {
        if value <= 10 {
            self.brightness = value;
        }
    }

    pub fn update<DELAY>(&mut self, delay: &mut DELAY)
    where
        DELAY: DelayNs,
    {
        
        for count in 0..10 {
            if count < self.brightness {
                self.pin.set_high().ok();
            } else {
                self.pin.set_low().ok();
            }
            delay.delay_ms(1); // Adjust for timing
        }
    }
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(96.MHz())
        .pll1_q_ck(48.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    // SPI4
    let sck = gpioe.pe12.into_alternate();
    let mosi = gpioe.pe14.into_alternate();

    let mut rst = gpioe.pe15.into_push_pull_output().forward();
    let mut dc = gpioe.pe13.into_push_pull_output().forward();
    let mut cs = gpioe.pe11.into_push_pull_output().forward();
    let mut led = Led::new(gpioe.pe10.into_push_pull_output().forward());
    

    // Initialise the SPI peripheral.
    let spi = dp
        .SPI4
        .spi(
            (sck, NoMiso, mosi),
            spi::MODE_0,
            3.MHz(),
            ccdr.peripheral.SPI4,
            &ccdr.clocks,
        )
        .forward();

    //let mut delay = cp.SYST.delay(ccdr.clocks).forward();
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, ccdr.clocks.sysclk().raw()).forward();

    led.update(&mut delay);
    

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut display = ST7735::new(
        spi_device,
        dc,
        rst,
        PixelOrder::BGR,
        true,
        Orientation::LandscapeSwapped,
        160,
        80,
    );
    display.init(&mut delay);
    display.set_orientation(Orientation::LandscapeSwapped); 
    display.clear(Rgb565::BLUE);

    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::RED)
        .background_color(Rgb565::GREEN)
        .build();

    Text::new(
        "This is a\nmultiline\nHello World!",
        Point::new(30, 30),
        style,
    )
    .draw(&mut display).unwrap();

    let logo = Bmp::from_slice(include_bytes!("rust.bmp")).unwrap();
    let lgogo = Image::new(&logo, Point::new(40, 0));
    logo.draw(&mut display).unwrap();

    loop {}
}
