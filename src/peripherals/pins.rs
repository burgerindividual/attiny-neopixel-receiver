use core::convert::Infallible;

use embedded_hal::digital::{ErrorType, OutputPin};

use crate::intrinsics::delay_cycles_short;
pub struct NoopPin;

impl ErrorType for NoopPin {
    type Error = Infallible;
}

impl OutputPin for NoopPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct CSNPin;

impl ErrorType for CSNPin {
    type Error = Infallible;
}

impl OutputPin for CSNPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let portb = crate::portb();
        portb.portb.modify(|_r, w| w.pb3().clear_bit());
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let portb = crate::portb();
        portb.portb.modify(|_r, w| w.pb3().set_bit());
        // approx 12us
        delay_cycles_short(192);
        Ok(())
    }
}
pub struct PB4;

impl ErrorType for PB4 {
    type Error = Infallible;
}

impl OutputPin for PB4 {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let portb = crate::portb();
        portb.portb.modify(|_r, w| w.pb4().clear_bit());
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let portb = crate::portb();
        portb.portb.modify(|_r, w| w.pb4().set_bit());
        Ok(())
    }
}
