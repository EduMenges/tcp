use std::time::Duration;

use midly::num::*;

use crate::{
    midi_action::MidiAction,
    text_to_midi::{Sheet, State},
};

const ONE_MINUTE_IN_MICROSECONDS: u32 = 60_000_000;

#[derive(Clone, Copy)]
/// Guarda um compasso.
pub struct TimeSignature {
    /// Numerador
    pub numerator: u8,
    /// Denominador
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
    /// Construtor a partir das informações cruas,
    /// onde numerador não é alterado, e denominador é um logaritmo de 2 da real nota.
    pub const fn from_raw(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator: 2_u8.pow(denominator as u32),
        }
    }
}

#[derive(Clone, Copy)]
/// Estrutura para guardar o estado usado na reprodução dos arquivos.
pub struct TimeState {
    /// Compasso
    time_signature: TimeSignature,
    /// Microsegundos por semimínima. Determina o tempo geral de reprodução das notas.
    microsecspqn: u24,
    /// Ticks por semimínima. Deve ser mutado somente na criação.
    pub tpqn: u15,
}

impl TimeState {
    /// Presume um BPM de 120.
    const D_MSPQN: u24 = Self::mspqn_from_bpm(State::D_BPM, 4);

    /// Define a quantidade de microsegundos por semimínima.
    ///
    /// Usado quando há mudanças no BPM.
    pub fn set_mspqn(&mut self, mspqn: u24) {
        self.microsecspqn = mspqn;
    }

    /// Getter para o compasso.
    pub const fn time_signature(self) -> TimeSignature {
        self.time_signature
    }

    /// Setter para o compasso.
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }

    /// Getter para o MSPQN.
    pub const fn mspqn(self) -> u24 {
        self.microsecspqn
    }

    ///
    pub const fn mspqn_from_bpm(bpm: u16, denominator: u32) -> u24 {
        u24::from_int_lossy(((ONE_MINUTE_IN_MICROSECONDS) * denominator) / (bpm * 4) as u32)
    }
    /// Sets the MSPQN based on a BPM
    pub fn set_mspqn_from_bpm(&mut self, bpm: u16) {
        self.set_mspqn(Self::mspqn_from_bpm(
            bpm,
            self.time_signature.denominator as _,
        ));
    }

    /// Getter para o BPM, utilizando o MSPQN.
    ///
    /// Para o cálculo, é ncessário ajustar de acordo com o denominador do compasso,
    /// pois o MSPQN é fixo para semimínimas.
    pub fn bpm(self) -> u16 {
        ((ONE_MINUTE_IN_MICROSECONDS as f64 / self.microsecspqn.as_int() as f64)
            * (self.time_signature().denominator as f64 / 4_f64)) as u16
    }

    /// A duração de um tick do MIDI.
    ///
    /// O cálculo é feito diretamente a partir do TPQN (definido no cabeçalho do arquivo)
    /// e do MSPQN.
    pub fn duration_per_tick(self) -> Duration {
        Duration::from_micros((self.mspqn().as_int() as u64) / (self.tpqn.as_int() as u64))
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            time_signature: TimeSignature::default(),
            microsecspqn: Self::D_MSPQN,
            tpqn: MidiAction::D_TPQN,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_bpm() {
        // Arrange
        let regular = TimeState::default();

        // Assert
        assert_eq!(regular.bpm(), 120);
    }

    #[test]
    fn mspqn_from_bpm() {
        // Arrange
        let mut regular = TimeState::default();

        // Act
        regular.set_mspqn_from_bpm(120);

        // Assert
        assert_eq!(regular.mspqn(), 500_000);
    }
}
