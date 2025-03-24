#![allow(dead_code)] // Suppress warnings about unused constants

use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::OutputPin;
use display_interface_spi::SPIInterface;

pub const SWRESET: u8 = 0x01;
pub const SLPOUT: u8 = 0x11;
pub const NORON: u8 = 0x13;
pub const INVOFF: u8 = 0x20;
pub const INVON: u8 = 0x21;
pub const DISPOFF: u8 = 0x28;
pub const DISPON: u8 = 0x29;
pub const CASET: u8 = 0x2A;
pub const RASET: u8 = 0x2B;
pub const RAMWR: u8 = 0x2C;
pub const RAMRD: u8 = 0x2E;
pub const COLMOD: u8 = 0x3A;
pub const MADCTL: u8 = 0x36;
pub const FRMCTR1: u8 = 0xB1;
pub const FRMCTR2: u8 = 0xB2;
pub const FRMCTR3: u8 = 0xB3;
pub const INVCTR: u8 = 0xB4;
pub const DISSET5: u8 = 0xB6;
pub const PWCTR1: u8 = 0xC0;
pub const PWCTR2: u8 = 0xC1;
pub const PWCTR3: u8 = 0xC2;
pub const PWCTR4: u8 = 0xC3;
pub const PWCTR5: u8 = 0xC4;

pub const PWCTR6: u8 = 0xFC;
pub const VMCTR1: u8 = 0xC5;
pub const GMCTRP1: u8 = 0xE0;
pub const GMCTRN1: u8 = 0xE1;


pub struct Command<'a> {
    pub instruction: u8,
    pub params: &'a [u8],
    pub delay_time: u32,
}

impl<'a> Command<'a> {
    pub(crate) fn new(instruction: u8, params: &'a [u8], delay_time: u32) -> Self {
        Self {
            instruction,
            params,
            delay_time,
        }
    }
}

