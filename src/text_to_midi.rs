use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::midi_action::MidiAction;

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
    Pause,
}

impl Distribution<Note> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Note {
        match rng.gen_range(0..=6) {
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
}

/// Estrutura que guarda o estado atual da música.
#[derive(Clone, Copy)]
pub struct State {
    /// BPM
    pub bpm: u16,
    /// O instrumento atual.
    pub instrument: u8,
    /// A oitava atual.
    pub octave: u8,
    /// O volume atual.
    pub volume: u16,
    /// A nota atual.
    pub note: Option<Note>,
}

impl State {
    /// A máxima oitava possível.
    pub const MAX_OCTAVE: u8 = 12;
    /// A oitava padrão.
    pub const DEFAULT_OCTAVE: u8 = 4;
    /// O volume padrão.
    pub const DEFAULT_VOLUME: u16 = 100;

    pub const MAX_VOLUME: u16 = u16::MAX >> 1;

    pub const DEFAULT_BPM: u16 = 120;

    pub const MAX_BPM: u16 = 360;

    /// Cria um estado novo.
    pub const fn new(instrument: u8, octave: u8, volume: u16, bpm: u16, note: Note) -> Self {
        Self {
            instrument,
            octave,
            volume,
            bpm,
            note: Some(note),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            instrument: 0,
            octave: Self::DEFAULT_OCTAVE,
            volume: Self::DEFAULT_VOLUME,
            bpm: Self::DEFAULT_BPM,
            note: Option::default(),
        }
    }
}

/// Segura informações sobre a música e oferece métodos para seu processamento.
pub struct Sheet {
    /// O BPM da partitura.
    bpm: u16,
    /// O estado atual.
    current_state: State,
    /// Os estados já processados.
    states: Vec<State>,
    /// O texto a ser processado.
    text: String,
}

impl Sheet {
    const D_R_PLUS: char = '東';
    const D_R_MINUS: char = '世';
    const D_BPM_PLUS: char = 'ß';

    /// Cria uma nova partitura a partir de uma BPM básica e um texto.
    pub fn new(bpm: u16, text: impl ToString) -> Self {
        Self {
            bpm,
            states: Vec::new(),
            text: text.to_string(),
            current_state: State::default(),
        }
    }

    /// Pegar o vetor com os estados e aplicar as mudanças conforme a especificação
    pub fn process(mut self) -> Vec<MidiAction> {
        self.process_text();
        let mut ret = Vec::<MidiAction>::new();

        self.current_state = self.states[0];
        ret.push(MidiAction::ChangeBPM(self.current_state.bpm));
        ret.push(MidiAction::ChangeInstrument(self.current_state.instrument));
        ret.push(MidiAction::ChangeVolume(self.current_state.volume));

        for actual_state in self.states {
            if actual_state.bpm != self.current_state.bpm {
                ret.push(MidiAction::ChangeBPM(actual_state.bpm));
            } else if actual_state.instrument != self.current_state.instrument {
                ret.push(MidiAction::ChangeInstrument(actual_state.instrument));
            } else if actual_state.volume != self.current_state.volume {
                ret.push(MidiAction::ChangeVolume(actual_state.volume));
            } else if let Some(note) = actual_state.note {
                match note {
                    Note::Pause => {
                        ret.push(MidiAction::Pause);
                    }
                    _ => {
                        ret.push(MidiAction::PlayNote(
                            (note as u8) + 12 * (actual_state.octave + 1),
                        ));
                    }
                }
            }

            self.current_state = actual_state;
        }

        ret
    }

    pub fn map_substring_to_char(&mut self) -> String {
        let text = self
            .text
            .replace("BPM+", &Self::D_BPM_PLUS.to_string())
            .replace("R+", &Self::D_R_PLUS.to_string())
            .replace("R-", &Self::D_R_MINUS.to_string());

        let mut aux = String::new();
        let mut prev_char = '\0';

        for c in text.chars() {
            if let Some(_new_note) = Note::from_char(prev_char) {
                if matches!(c, 'o' | 'O' | 'I' | 'i' | 'u' | 'U') {
                    aux.push(prev_char);
                    prev_char = c;
                    continue;
                }
            }
            aux.push(c);
            prev_char = c;
        }

        aux
    }

    pub fn process_text(&mut self) {
        let text = self.map_substring_to_char();

        for c in text.chars() {
            self.parse_char(c);
        }
    }

    /// Altera o `current_state` e coloca no fim do vetor
    fn parse_char(&mut self, ch: char) {
        // ABCDEFG
        let new_note: Option<Note> = Note::from_char(ch);
        if let Some(note) = new_note {
            self.current_state.note = Some(note);
        } else {
            self.current_state.note = None;
            match ch {
                '+' => {
                    // Aumenta volume para o DOBRO do volume; Se não puder aumentar, volta ao volume default (de início)
                    self.current_state.volume = if self.current_state.volume * 2 > State::MAX_VOLUME
                    {
                        State::MAX_VOLUME
                    } else {
                        self.current_state.volume * 2
                    };
                }
                '-' => {
                    // Volume retorna ao volume padrão
                    self.current_state.volume = State::DEFAULT_VOLUME;
                }
                'o' | 'O' | 'I' | 'i' | 'u' | 'U' => {
                    // Nesse caso, caso que em que não há uma nota anterior, altera o instrumento para o telefone
                    self.current_state.instrument = 125;
                    self.current_state.note = Some(Note::Do);
                    let aux_state = *self.states.last().unwrap();
                    self.states.push(self.current_state);
                    self.current_state = aux_state;
                }

                //R+
                Self::D_R_PLUS => {
                    // Aumenta UMA oitava; Se não puder, aumentar, volta à oitava default (de início)
                    let new_octave = self.current_state.octave + 1;
                    self.current_state.octave = if new_octave > State::MAX_OCTAVE {
                        State::DEFAULT_OCTAVE
                    } else {
                        new_octave
                    };
                }

                //R-
                Self::D_R_MINUS => {
                    // Diminui UMA oitava;
                    self.current_state.octave = self.current_state.octave.saturating_sub(1);
                }
                //BPM+
                Self::D_BPM_PLUS => {
                    // Aumenta BPM em 80 unidades
                    self.current_state.bpm = self.current_state.bpm.saturating_add(80);
                }

                '?' => {
                    //Toca uma nota aleatória (de A a G), randomicamente escolhida
                    let mut rng = rand::thread_rng();
                    let random_note: Note = rng.gen();
                    self.current_state.note = Some(random_note);
                }

                '\n' => {
                    //Trocar instrumento aleatorio
                    let mut rng = rand::thread_rng();
                    self.current_state.instrument = rng.gen_range(0..=i8::MAX as u8);
                }

                ';' => {
                    //Atribui valor aleatorio ao BPM
                    let mut rng = rand::thread_rng();
                    self.current_state.bpm = rng.gen_range(1..State::MAX_BPM);
                }
                _ => {}
            }
        }

        self.states.push(self.current_state);
    }
}

#[cfg(test)]
mod test {
    use super::{Sheet, State};

    #[test]
    fn match_process_general_text_behavior() {
        let text = "; \nasBPM+ ?!;-+".to_string();
        let mut sheet = Sheet::new(State::DEFAULT_BPM, text);
        let received_text = sheet.map_substring_to_char();

        let expected_text = "; \nasß ?!;-+".to_string();

        assert_eq!(expected_text, received_text);
    }

    #[test]
    fn match_process_note_text_behavior() {
        let text = "AaBbCcDdEeFfGg".to_string();
        let mut sheet = Sheet::new(State::DEFAULT_BPM, text);
        let received_text = sheet.map_substring_to_char();

        let expected_text = "AaBbCcDdEeFfGg".to_string();

        assert_eq!(expected_text, received_text);
    }

    #[test]
    fn match_process_substring_text_behavior() {
        let text = "BPM+R+R-".to_string();
        let mut sheet = Sheet::new(State::DEFAULT_BPM, text);
        let received_text = sheet.map_substring_to_char();

        let mut expected_text = Sheet::D_BPM_PLUS.to_string();
        expected_text.push(Sheet::D_R_PLUS);
        expected_text.push(Sheet::D_R_MINUS);

        assert_eq!(expected_text, received_text);
    }

    #[test]
    fn match_process_vogals_text_behavior() {
        let text = "OoIiUuAiBICuDUEoFo".to_string();
        let mut sheet = Sheet::new(State::DEFAULT_BPM, text);
        let received_text = sheet.map_substring_to_char();

        let expected_text = "OoIiUuAABBCCDDEEFF".to_string();

        assert_eq!(expected_text, received_text);
    }
}
