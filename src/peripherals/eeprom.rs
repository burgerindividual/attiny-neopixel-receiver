use avr_device::attiny85::EEPROM;

use crate::peripherals;

pub fn write(address: u16, new_value: u8) {
    flush();

    let mut raw_eeprom = peripherals::eeprom();

    raw_eeprom.eear.write(|w| w.bits(address));

    // Read the previous value to create a difference mask with the value to be
    // written
    raw_eeprom.eecr.write(|w| w.eere().set_bit());
    let previous_value = raw_eeprom.eedr.read().bits();

    // exit early if the we have the same value already stored
    if previous_value == new_value {
        return;
    }

    // special condition where we can directly do an erase
    if new_value == 0xFF {
        raw_eeprom
            .eecr
            .write(|w| w.eepm().erase().eempe().set_bit().eepe().clear_bit());
        raw_eeprom
            .eecr
            .write(|w| w.eepm().erase().eempe().set_bit().eepe().set_bit());
    } else {
        raw_eeprom.eedr.write(|w| w.bits(new_value));
        // case where byte can be programmed without erasing
        if (previous_value & new_value) == new_value {
            raw_eeprom
                .eecr
                .write(|w| w.eepm().write().eempe().set_bit().eepe().clear_bit());
            raw_eeprom
                .eecr
                .write(|w| w.eepm().write().eempe().set_bit().eepe().set_bit());
        } else {
            // final case, where both an erase and a program is required
            raw_eeprom
                .eecr
                .write(|w| w.eepm().atomic().eempe().set_bit().eepe().clear_bit());
            raw_eeprom
                .eecr
                .write(|w| w.eepm().atomic().eempe().set_bit().eepe().set_bit());
        }
    }
}

pub fn read(address: u16) -> u8 {
    flush();

    let mut raw_eeprom = peripherals::eeprom();

    raw_eeprom.eear.write(|w| w.bits(address));
    // The write will also clear the EEPE bit, which will disable writing
    raw_eeprom.eecr.write(|w| w.eere().set_bit());

    raw_eeprom.eedr.read().bits()
}

fn flush() {
    while peripherals::eeprom().eecr.read().eepe().bit_is_set() {}
}
