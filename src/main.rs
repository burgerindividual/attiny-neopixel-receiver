#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]
#![allow(dead_code)]
#![allow(unused)]

mod intrinsics;
mod packet;
mod peripherals;

use core::arch::asm;
use core::hint::black_box;
use core::mem::transmute;
use core::panic::PanicInfo;
use core::ptr;
use core::sync::atomic::compiler_fence;

use avr_device::asm::{delay_cycles, nop};
use avr_device::attiny85::PORTB;
use avr_device::interrupt::CriticalSection;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;
use embedded_nrf24l01::{
    Configuration, CrcMode, DataRate, Device, Payload, StandbyMode, NRF24L01, PIPES_COUNT,
};
use peripherals::led::{hue_cycle, write_led, LEDColor};
use peripherals::pins::{CSNPin, NoopPin, PB4};
use peripherals::spi::Spi;

use crate::peripherals::*;

// const GAMMA8: [u8; 256] = [
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// 0, 0, 0, 0, 1, 1, 1, 1,     1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2,
// 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,     5, 6, 6, 6, 6, 7, 7, 7,
// 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
//     14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23,
// 24, 24, 25, 25, 26, 27,     27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35,
// 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,     47, 48, 49, 50, 50, 51,
// 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
//     73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96,
// 98, 99, 101, 102, 104,     105, 107, 109, 110, 112, 114, 115, 117, 119, 120,
// 122, 124, 126, 127, 129, 131, 133, 135, 137,     138, 140, 142, 144, 146,
// 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
//     177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208,
// 210, 213, 215, 218, 220,     223, 225, 228, 231, 233, 236, 239, 241, 244,
// 247, 249, 252, 255, ];

pub const LED_STRIP_LENGTH: u16 = 375;

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    eeprom::write(
        511, // last byte in attiny85 eeprom
        0b10101010,
    );

    let portb = peripherals::portb();
    loop {
        delay_cycles(10000);
        portb.pinb.modify(|_, w| w.pb4().set_bit());
    }
}

#[no_mangle]
pub extern "C" fn main() {
    // clear panic byte
    eeprom::write(
        511, // last byte in attiny85 eeprom
        0xFF,
    );

    let portb = peripherals::portb();
    portb.ddrb.write(|w| {
        w
            .pb0() // MOSI
            .clear_bit()
            .pb1() // MISO
            .set_bit()
            .pb2() // SCK
            .set_bit()
            .pb3() // NCS
            .set_bit()
            .pb4() // LED Data Pin
            .set_bit()
    });

    // enable internal pullup resistor for miso
    portb.portb.write(|w| w.pb1().set_bit().pb3().set_bit());

    // let tc0 = peripherals::tc0();
    // tc0.tccr0a.modify(|_r, w| w.wgm0().ctc());
    // // 250KHz at 16MHz core clock
    // tc0.tccr0b.modify(|_r, w| w.cs0().prescale_64());

    // // reset prescaler and counter
    // tc0.gtccr.modify(|_r, w| w.psr0().set_bit());
    // tc0.tcnt0.write(|w| w.bits(0));

    // // compare once per tick
    // tc0.ocr0a.write(|w| w.bits(1));

    // // start the timer
    // tc0.tifr.write(|w| w.ocf0a().set_bit());
    // tc0.timsk.modify(|_r, w| w.ocie0a().set_bit());

    let mut spi = Spi::new();

    // CE is set to noop because it's always tied to RX.
    let mut nrf = NRF24L01::new(NoopPin, CSNPin, spi).unwrap();

    nrf.set_rf(&DataRate::R250Kbps, 2).unwrap();
    nrf.set_frequency(13).unwrap();
    nrf.set_address_width(3).unwrap();
    nrf.set_crc(CrcMode::OneByte).unwrap();
    nrf.set_rx_addr(0, b"nrf").unwrap();
    nrf.set_auto_ack(&[false; PIPES_COUNT]).unwrap();
    nrf.set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    nrf.set_pipes_rx_lengths(&[None; PIPES_COUNT]).unwrap();
    nrf.set_interrupt_mask(false, true, true).unwrap();
    nrf.clear_interrupts().unwrap();

    let mut nrf_rx = nrf.rx().unwrap();
    // 130us transition from standby mode, 2080 cycles at 16mhz
    delay_cycles(2080);

    let mut led_toggle = false;
    let mut start_color = LEDColor::from_rgb(0xFF, 0x00, 0x00);

    loop {
        // process if RX ready interrupt is set
        if nrf_rx.get_interrupts().unwrap().0 {
            let payload = nrf_rx.read().unwrap();
            nrf_rx.clear_interrupts().unwrap();

            let decoded = core::str::from_utf8(&payload).unwrap();

            if decoded == "Ping!" {
                led_toggle = !led_toggle;
            } else {
                panic!();
            }
        }

        if led_toggle {
            let mut current_color = start_color;
            for _ in 0..LED_STRIP_LENGTH {
                write_led::<4>(&portb, current_color);
                hue_cycle(&mut current_color, 1);
            }
        } else {
            for _ in 0..LED_STRIP_LENGTH {
                write_led::<4>(&portb, LEDColor::BLACK);
            }
        }

        hue_cycle(&mut start_color, 1);

        // waiting for atleast 250μs ends the led line commands. we'll wait for 300μs
        delay_cycles(4800);
        // delay for 2ms, this is the maximum refresh speed of the led strip
        // (this can be shorter)
        // delay_cycles(16000 * 2);
    }
}
