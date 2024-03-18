#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LEDStripOption {
    Enabled = 0,
    Twinkle = 1,
    HueCycle = 2,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct LEDStripState(u8);

impl LEDStripState {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(0b111);

    pub const fn from(packed: u8) -> Self {
        Self(packed)
    }

    pub const fn single(direction: LEDStripOption) -> Self {
        Self(1 << direction as u8)
    }

    pub fn add(&mut self, dir: LEDStripOption) {
        self.0 |= 1 << dir as u8;
    }

    pub fn add_all(&mut self, set: LEDStripState) {
        self.0 |= set.0;
    }

    pub fn remove(&mut self, dir: LEDStripOption) {
        self.0 &= !(1 << dir as u8);
    }

    pub const fn contains(&self, dir: LEDStripOption) -> bool {
        (self.0 & (1 << dir as u8)) != 0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
