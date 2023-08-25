use std::time::Duration;

use midly::num::*;

use crate::midi_action::MIDIaction;

const ONE_MINUTE_IN_MICROSECONDS: u32 = 60_000_000;

#[derive(Clone, Copy)]
pub struct TimeState {
    numerator: u8,
    denominator: u8,
    mspqn: u24,
    ppqn: u16,
}

impl TimeState {
    const D_MSPQN: u24 = u24::from_int_lossy(500_000);

    pub fn set_signature(&mut self, numerator: u8, denominator: u8) {
        self.denominator = 2_u8.pow(denominator as u32);
        self.numerator = numerator;
    }

    pub fn set_mspqn(&mut self, mspqn: u24) {
        self.mspqn = mspqn;
    }

    pub const fn time_signature(&self) -> (u8, u8) {
        (self.numerator, self.denominator)
    }

    pub const fn mspqn(&self) -> u24 {
        self.mspqn
    }

    pub fn bpm(&self) -> u16 {
        ((ONE_MINUTE_IN_MICROSECONDS as f64 / self.mspqn.as_int() as f64)
            * (self.denominator as f64 / 4_f64)) as u16
    }

    pub fn duration_per_tick(&self) -> Duration {
        Duration::from_micros((self.mspqn().as_int() as u64) / (self.ppqn as u64))
    }

    pub fn set_mspqn_from_bpm(&mut self, bpm: u16) {
        self.set_mspqn(u24::from_int_lossy(
            ((ONE_MINUTE_IN_MICROSECONDS * self.denominator as u32) as f64 / (bpm * 4) as f64)
                as u32,
        ));
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
            mspqn: Self::D_MSPQN,
            ppqn: MIDIaction::D_PPQN,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bpm() {
        // Arrange
        let regular = TimeState::default();

        // Assert
        assert_eq!(regular.bpm(), 120);
    }

    #[test]
    fn mspqn() {
        // Arrange
        let mut regular = TimeState::default();
        
        // Act
        regular.set_mspqn_from_bpm(120);

        // Assert
        assert_eq!(regular.mspqn(), TimeState::D_MSPQN);
    }
}