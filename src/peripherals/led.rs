use core::arch::asm;

use avr_device::attiny85::PORTB;

#[derive(Clone, Copy)]
pub struct LEDColor {
    grb: [u8; 3],
}

impl LEDColor {
    pub const BLACK: Self = Self::from_rgb(0x00, 0x00, 0x00);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { grb: [g, r, b] }
    }

    pub const fn as_grb(&self) -> &[u8; 3] {
        &self.grb
    }

    pub fn g(&mut self) -> &mut u8 {
        &mut self.grb[0]
    }

    pub fn r(&mut self) -> &mut u8 {
        &mut self.grb[1]
    }

    pub fn b(&mut self) -> &mut u8 {
        &mut self.grb[2]
    }
}

pub fn write_led<const PIN: u8>(portb: &PORTB, color: LEDColor) {
    for color in color.as_grb() {
        for bit in (0..u8::BITS).rev() {
            let cur_bit_set = (color & (0b1 << bit)) != 0;

            if cur_bit_set {
                unsafe {
                    asm!(
                        "sbi 24, {pin}",
                        ".rept 12",
                        "nop",
                        ".endr",
                        "cbi 24, {pin}",
                        ".rept 7",
                        "nop",
                        ".endr",
                        pin = const { PIN }
                    );
                }
            } else {
                unsafe {
                    asm!(
                        "sbi 24, {pin}",
                        ".rept 4",
                        "nop",
                        ".endr",
                        "cbi 24, {pin}",
                        ".rept 12",
                        "nop",
                        ".endr",
                        pin = const { PIN }
                    );
                }
            }
        }
    }
}

pub fn hue_cycle(color: &mut LEDColor, speed: u8) {
    *color.g() = color
        .g()
        .saturating_add(if *color.r() == 0xFF && *color.b() == 0x00 {
            speed
        } else {
            0
        });
    *color.g() = color
        .g()
        .saturating_sub(if *color.b() == 0xFF && *color.r() == 0x00 {
            speed
        } else {
            0
        });

    *color.b() = color
        .b()
        .saturating_add(if *color.g() == 0xFF && *color.r() == 0x00 {
            speed
        } else {
            0
        });
    *color.b() = color
        .b()
        .saturating_sub(if *color.r() == 0xFF && *color.g() == 0x00 {
            speed
        } else {
            0
        });

    *color.r() = color
        .r()
        .saturating_add(if *color.b() == 0xFF && *color.g() == 0x00 {
            speed
        } else {
            0
        });
    *color.r() = color
        .r()
        .saturating_sub(if *color.g() == 0xFF && *color.b() == 0x00 {
            speed
        } else {
            0
        });
}
