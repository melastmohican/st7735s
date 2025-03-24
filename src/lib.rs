//! This crate provides a ST7735 driver to connect to TFT displays.
#![no_std]

use crate::cmd::{
    Command, CASET, COLMOD, DISPON, FRMCTR1, FRMCTR2, FRMCTR3, GMCTRN1, GMCTRP1, INVCTR, INVOFF,
    INVON, MADCTL, NORON, PWCTR1, PWCTR2, PWCTR3, PWCTR4, PWCTR5, PWCTR6, RAMWR, RASET, SLPOUT,
    SWRESET, VMCTR1,
};
use display_interface::{DataFormat, WriteOnlyDataCommand};
use display_interface_spi::SPIInterface;
use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::pixelcolor::raw::{RawU16, ToBytes};
use embedded_graphics::{draw_target::DrawTarget, geometry::Size, pixelcolor::Rgb565, Pixel};
use embedded_graphics::pixelcolor::IntoStorage;
use embedded_graphics::prelude::RawData;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

mod cmd;

/// Display Pixel Color Mode
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PixelOrder {
    /// Red, Green, Blue,
    RGB = 0x00,
    /// Blue, Green, Red,
    BGR = 0x08,
}

/// Display orientation.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Orientation {
    Portrait = 0x00,
    Landscape = 0x60,
    PortraitSwapped = 0xC0,
    LandscapeSwapped = 0xA0,
}
pub struct ST7735<SPI, DC, RST> {
    interface: SPIInterface<SPI, DC>,
    rst: RST,
    /// Whether the display is RGB (true) or BGR (false)
    pixel_order: PixelOrder,

    /// Whether the colours are inverted (true) or not (false)
    inverted: bool,

    orientation: Orientation,

    /// Global image offset
    offset: Size,
    size: Size
}

impl<SPI, DC, RST, E> ST7735<SPI, DC, RST>
where
    SPI: SpiDevice<Error = E>,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        pixel_order: PixelOrder,
        inverted: bool,
        orientation: Orientation,
        width: u32,
        height: u32,
    ) -> Self {
        let interface = SPIInterface::new(spi, dc);
        Self {
            interface,
            rst,
            pixel_order,
            inverted,
            orientation,
            offset: Size::new(1, 26),
            size: Size::new(width, height)
        }
    }

    pub fn reset<DELAY>(&mut self, delay: &mut DELAY)
    where
        DELAY: DelayNs,
    {
        self.rst.set_high().ok();
        delay.delay_ms(10);
        self.rst.set_low().ok();
        delay.delay_ms(10);
        self.rst.set_high().ok();
    }

    pub fn send_init_commands<DELAY>(&mut self, delay: &mut DELAY)
    where
        DELAY: DelayNs,
    {
        let madctl = &[self.orientation as u8 | self.pixel_order as u8];
        let caset = &[
            0x00,
            self.offset.width as u8,
            0x00,
            (self.size.width + self.offset.width) as u8,
        ];
        let raset = &[
            0x00,
            self.offset.height as u8,
            0x00,
            (self.size.height + self.offset.height ) as u8,
        ];

        let init_sequence = [
            Command::new(SWRESET, &[], 120),
            Command::new(SLPOUT, &[], 255),
            Command::new(FRMCTR1, &[0x01, 0x2C, 0x2D], 0),
            Command::new(FRMCTR2, &[0x01, 0x2C, 0x2D], 0),
            Command::new(FRMCTR3, &[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D], 10),
            Command::new(INVCTR, &[0x07], 0),
            Command::new(PWCTR1, &[0xA2, 0x02, 0x84], 0),
            Command::new(PWCTR2, &[0xC5], 0),
            Command::new(PWCTR3, &[0x0A, 0x00], 0),
            Command::new(PWCTR4, &[0x8A, 0x2A], 0),
            Command::new(PWCTR5, &[0x8A, 0xEE], 0),
            Command::new(VMCTR1, &[0x0E], 0),
            Command::new(if self.inverted { INVON } else { INVOFF }, &[], 0),
            Command::new(MADCTL, madctl, 0),
            Command::new(COLMOD, &[0x05], 0),
            Command::new(CASET, caset, 0),
            Command::new(RASET, raset, 0),
            Command::new(
                GMCTRP1,
                &[
                    0x02, 0x1c, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2d, 0x29, 0x25, 0x2b, 0x39, 0x00,
                    0x01, 0x03, 0x10,
                ],
                0,
            ),
            Command::new(
                GMCTRN1,
                &[
                    0x03, 0x1d, 0x07, 0x06, 0x2e, 0x2c, 0x29, 0x2d, 0x2e, 0x2e, 0x37, 0x3f, 0x00,
                    0x00, 0x02, 0x10,
                ],
                0,
            ),
            Command::new(NORON, &[], 10),
            Command::new(DISPON, &[], 100),
        ];

        for Command {
            instruction,
            params,
            delay_time,
        } in init_sequence
        {
            let _ = self.interface.send_commands(DataFormat::U8(&[instruction]));
            if !params.is_empty() {
                let _ = self.interface.send_data(DataFormat::U8(params));
            }
            if delay_time > 0 {
                delay.delay_ms(delay_time);
            }
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY)
    where
        DELAY: DelayNs,
    {
        self.reset(delay);
        self.send_init_commands(delay);
    }

    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), ()> {
        self.interface.send_commands(DataFormat::U8(&[MADCTL]));
        self.interface.send_data(DataFormat::U8(
            &[orientation as u8 | self.pixel_order as u8],
        ));

        self.orientation = orientation;
        Ok(())
    }

    /// Sets the global offset of the displayed image
    pub fn set_offset(&mut self, dx: u16, dy: u16) {
        self.offset = Size::new(dx as u32, dy as u32);
    }

    /// Sets the address window for the display.
    pub fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) {
        let mut window_loc_data: [u8; 4] = [0; 4];
        self.interface.send_commands(DataFormat::U8(&[CASET]));
        window_loc_data[0] = self.offset.width as u8; 
        window_loc_data[1] = (self.offset.width + sx as u32) as u8;
        window_loc_data[2] = self.offset.width as u8;
        window_loc_data[3] = (self.offset.width + ex as u32) as u8;
        self.interface.send_data(DataFormat::U8(&window_loc_data));
        self.interface.send_commands(DataFormat::U8(&[RASET]));
        window_loc_data[0] = self.offset.height as u8;
        window_loc_data[1] = (self.offset.height + sy as u32) as u8;
        window_loc_data[2] = self.offset.height as u8;
        window_loc_data[3] = (self.offset.height + ey as u32) as u8;
        self.interface.send_data(DataFormat::U8(&window_loc_data));
        self.interface.send_commands(DataFormat::U8(&[RAMWR]));
    }

    /// Sets a pixel color at the given coords.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: RawU16) {
        self.set_address_window(x, y, x, y);
        self.interface.send_data(DataFormat::U8(&color.into_inner().to_be_bytes()));
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.set_address_window(0, 0, (self.size.width - 1) as u16, (self.size.height - 1) as u16);
        for _ in 0..(self.size.width * self.size.height) {
            self.interface.send_data(DataFormat::U8(color.to_be_bytes().as_ref()));
        }
    }
}

impl<SPI, DC, RST, E> OriginDimensions for ST7735<SPI, DC, RST>
where
    DC: OutputPin,
    RST: OutputPin,
    SPI: SpiDevice<Error = E>,
{
    fn size(&self) -> Size {
        Size::new(self.size.width, self.size.height)
    }
}

impl<SPI, DC, RST, E> DrawTarget for ST7735<SPI, DC, RST>
where
    SPI: SpiDevice<Error = E>,
    DC: OutputPin,
    RST: OutputPin,
{
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            // Only draw pixels that would be on screen
            //if point.x >= 0 && point.y >= 0 && point.x < self.size.width as i32 && point.y < self.size. height as i32
            //{
                self.set_pixel(point.x as u16, point.y as u16, RawU16::from(color));
            //}
        }
        Ok(())
    }
}
