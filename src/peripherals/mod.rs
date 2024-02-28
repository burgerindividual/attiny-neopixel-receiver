use core::mem::transmute;

use avr_device::attiny85::*;

pub mod eeprom;
pub mod pins;
pub mod spi;

pub const fn ac() -> AC {
    unsafe { transmute(()) }
}

pub const fn adc() -> ADC {
    unsafe { transmute(()) }
}

pub const fn boot_load() -> BOOT_LOAD {
    unsafe { transmute(()) }
}

pub const fn cpu() -> CPU {
    unsafe { transmute(()) }
}

pub const fn eeprom() -> EEPROM {
    unsafe { transmute(()) }
}

pub const fn exint() -> EXINT {
    unsafe { transmute(()) }
}

pub const fn fuse() -> FUSE {
    unsafe { transmute(()) }
}

pub const fn lockbit() -> LOCKBIT {
    unsafe { transmute(()) }
}

pub const fn portb() -> PORTB {
    unsafe { transmute(()) }
}

pub const fn tc0() -> TC0 {
    unsafe { transmute(()) }
}

pub const fn tc1() -> TC1 {
    unsafe { transmute(()) }
}

pub const fn usi() -> USI {
    unsafe { transmute(()) }
}

pub const fn wdt() -> WDT {
    unsafe { transmute(()) }
}
