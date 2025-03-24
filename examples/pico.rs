//! Draw a image. On a 80x1604 ST7735 display over SPI.
//!
//! This example is for the Raspberry Pi Pico board using SPI0.
//!
//! Wiring connections are as follows for  display:
//!
//! ```
//! LCD | => | Pico
//! ----|----|-----
//! VCC | -> | VSYS
//! GND | -> | GND
//! DIN | -> | 19
//! CLK | -> | 18
//! CS | -> | 9
//! DC | -> | 8
//! RST | -> | 12
//! BL | -> | 13
//! ```
//!
//! Run on a Rpi Pico with `cargo run --example st7735s`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use bsp::entry;

use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog};
use cortex_m::asm::nop;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{
    image::{Image, ImageRaw},
    prelude::*,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use rp_pico as bsp;
use rp_pico::hal::fugit::RateExtU32;
use rp_pico::hal::gpio::FunctionSpi;
use rp_pico::hal::{spi, Clock};
use tinybmp::Bmp;
use st7735s::{Orientation, PixelOrder, ST7735};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    //let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut delay = DelayCompat(cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    ));

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sck = pins.gpio18.into_function::<FunctionSpi>(); // SCK
    let mosi = pins.gpio19.into_function::<FunctionSpi>(); // SCL TX
    let miso = pins.gpio16.into_function::<FunctionSpi>(); // SDA RX

    let dc = pins.gpio8.into_push_pull_output();
    let cs = pins.gpio9.into_push_pull_output();
    let rst = pins.gpio12.into_push_pull_output();
    let mut bl = pins.gpio13.into_push_pull_output();

    // Create an SPI driver instance for the SPI0 device
    let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI0, (mosi, miso, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut display = ST7735::new(
        spi_dev,
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

    // draw rust logo
    let logo = Bmp::from_slice(include_bytes!("rust.bmp")).unwrap();
    let logo = Image::new(&logo, Point::new(40, 0));
    logo.draw(&mut display).unwrap();

    bl.set_high().ok();

    loop {
        nop()
    }
}

struct DelayCompat(cortex_m::delay::Delay);
impl embedded_hal::delay::DelayNs for DelayCompat {
    fn delay_ns(&mut self, mut ns: u32) {
        while ns > 1000 {
            self.0.delay_us(1);
            ns = ns.saturating_sub(1000);
        }
    }

    fn delay_us(&mut self, us: u32) {
        self.0.delay_us(us);
    }
}
