use std::time::Duration;

use midly::num::*;

use crate::midi_action::MIDIaction;

const ONE_MINUTE_IN_MICROSECONDS: u32 = 60_000_000;

#[derive(Clone, Copy)]
pub struct TimeSignature {
    /// Numerator of time signature
    pub numerator: u8,
    /// Denominator of time signature
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

impl TimeSignature {
    /**
    Builds from raw, where `numerator` is the numerator,
    `denominator` is a power of 2.
    */
    pub const fn from_raw(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator: 2_u8.pow(denominator as u32),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TimeState {
    /// The time signature
    time_signature: TimeSignature,
    /// Microseconds per quarter note
    microsecspqn: u24,
    /// Ticks per quarter note (should only be mutated at creation)
    pub tpqn: u15,
}

impl TimeState {
    const D_MSPQN: u24 = u24::from_int_lossy(500_000);

    /// Sets the microseconds per quarter note
    pub fn set_mspqn(&mut self, mspqn: u24) {
        self.microsecspqn = mspqn;
    }

    /// Getter for time signature
    pub const fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    /// Getter for the microseconds per quarter note
    pub const fn mspqn(&self) -> u24 {
        self.microsecspqn
    }

    /// Getter for the BPM
    pub fn bpm(&self) -> u16 {
        ((ONE_MINUTE_IN_MICROSECONDS as f64 / self.microsecspqn.as_int() as f64)
            * (self.time_signature().denominator as f64 / 4_f64)) as u16
    }


    pub fn duration_per_tick(&self) -> Duration {
        Duration::from_micros((self.mspqn().as_int() as u64) / (self.tpqn.as_int() as u64))
    }

    pub fn mspqn_from_bpm(bpm: u16, denominator: u32) -> u24 {
        u24::from_int_lossy(
            (((ONE_MINUTE_IN_MICROSECONDS) * denominator) as f64 / (bpm * 4) as f64) as u32
        )
    }
    /// Sets the MSPQN based on a BPM
    pub fn set_mspqn_from_bpm(&mut self, bpm: u16) {
        self.set_mspqn(Self::mspqn_from_bpm(bpm, self.time_signature.denominator as _));
    }

    /// Sets the time signature
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            time_signature: Default::default(),
            microsecspqn: Self::D_MSPQN,
            tpqn: MIDIaction::D_TPQN,
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
