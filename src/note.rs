use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/// Enum com as notas possíveis.
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum Note {
    /// Nota dó.
    #[default]
    Do = 0,
    /// Nota ré.
    Re = 2,
    /// Nota mi.
    Mi = 4,
    /// Nota fa.
    Fa = 5,
    /// Nota sol.
    Sol = 7,
    /// Nota la.
    La = 9,
    /// Nota si.
    Si = 11,
    /// Nota de pausa.
    Pause = 13,
}

impl Distribution<Note> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Note {
        match rng.gen_range(0..=Note::TOTAL_NOTES) {
            0 => Note::Do,
            1 => Note::Re,
            2 => Note::Mi,
            3 => Note::Fa,
            4 => Note::Sol,
            5 => Note::La,
            _ => Note::Si,
        }
    }
}

impl Note {
    /// Descontando os acidentes que não estão na especificação do trabalho.
    pub const TOTAL_NOTES: usize = 8;

    /// A partir de um caractere, cria uma nota.
    pub const fn from_char(ch: char) -> Option<Self> {
        match ch {
            'A' | 'a' => Some(Self::La),
            'B' | 'b' => Some(Self::Si),
            'C' | 'c' => Some(Self::Do),
            'D' | 'd' => Some(Self::Re),
            'E' | 'e' => Some(Self::Mi),
            'F' | 'f' => Some(Self::Fa),
            'G' | 'g' => Some(Self::Sol),
            ' ' => Some(Self::Pause),
            _ => None,
        }
    }

    /// Calcula o valor da nota para a reprodução em MIDI.
    pub const fn to_midi(self, octave: u8) -> u8 {
        self as u8 + 12 * (1 + octave)
    }
}
