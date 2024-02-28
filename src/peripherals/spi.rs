use core::arch::asm;
use core::hint::black_box;

use avr_device::asm::delay_cycles;
use avr_device::attiny85::USI;
use embedded_hal::spi::SpiBus;
use embedded_hal_nb::spi::{ErrorKind, ErrorType, FullDuplex};
use nb::Error::WouldBlock;

use crate::intrinsics::delay_cycles_short;
use crate::peripherals;

pub struct Spi {
    needs_flush: bool,
}

impl Spi {
    pub fn new() -> Self {
        let raw_usi = peripherals::usi();
        raw_usi.usicr.write(|w| {
            w.usiwm()
                .three_wire()
                .usisie()
                .clear_bit()
                .usioie()
                .clear_bit()
                .usics()
                .no_clock()
                .usitc()
                .clear_bit()
                .usiclk()
                .clear_bit()
        });
        raw_usi.usidr.write(|w| w.bits(0));

        Self { needs_flush: false }
    }
}

impl ErrorType for Spi {
    type Error = ErrorKind;
}

impl SpiBus for Spi {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let raw_usi = peripherals::usi();

        if words.len() > 0 {
            self.flush()?;
        }

        for i in 0..words.len() {
            raw_usi.usidr.write(|w| w.bits(0x00));
            self.needs_flush = true;
            self.flush()?;
            words[i] = raw_usi.usidr.read().bits();
        }

        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let raw_usi = peripherals::usi();

        for i in 0..words.len() {
            self.flush()?;
            raw_usi.usidr.write(|w| w.bits(words[i]));
            self.needs_flush = true;
        }

        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        let raw_usi = peripherals::usi();

        let read_write_len = read.len().min(write.len());

        if read_write_len > 0 {
            self.flush()?;
        }

        for i in 0..read_write_len {
            raw_usi.usidr.write(|w| w.bits(write[i]));
            self.needs_flush = true;
            self.flush()?;
            read[i] = raw_usi.usidr.read().bits();
        }

        if read.len() > write.len() {
            let read_len = read.len();
            let trailing_read = &mut read[read_write_len..read_len];

            for i in 0..trailing_read.len() {
                raw_usi.usidr.write(|w| w.bits(0x00));
                self.needs_flush = true;
                self.flush()?;
                trailing_read[i] = raw_usi.usidr.read().bits();
            }
        } else {
            let write_len = write.len();
            let trailing_write = &write[read_write_len..write_len];

            for i in 0..trailing_write.len() {
                self.flush()?;
                raw_usi.usidr.write(|w| w.bits(trailing_write[i]));
                self.needs_flush = true;
            }
        }

        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let raw_usi = peripherals::usi();

        if words.len() > 0 {
            self.flush()?;
        }

        for i in 0..words.len() {
            raw_usi.usidr.write(|w| w.bits(words[i]));
            self.needs_flush = true;
            self.flush()?;
            words[i] = raw_usi.usidr.read().bits();
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        if self.needs_flush {
            let raw_usi = peripherals::usi();

            raw_usi.usisr.write(|w| w.usioif().set_bit());

            while raw_usi.usisr.read().usioif().bit_is_clear() {
                // done to restrict spi speed to ~250kbps
                delay_cycles_short(16);
                raw_usi.usicr.write(|w| {
                    w.usiwm()
                        .three_wire()
                        .usisie()
                        .clear_bit()
                        .usioie()
                        .clear_bit()
                        .usics()
                        .ext_pos()
                        .usitc()
                        .set_bit()
                        .usiclk()
                        .set_bit()
                });
            }

            self.needs_flush = false;
        }

        Ok(())
    }
}
