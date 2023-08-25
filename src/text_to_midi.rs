use std::ops::Add;

use midi_msg::MidiMsg;
use midly::{num::*, Header, MetaMessage, Smf, Track, TrackEvent};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::midi_action::MIDIaction;

/// Enum com as notas possíveis.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Note {
    /// Nota dó.
    Do,
    /// Nota ré.
    Re,
    /// Nota mi.
    Mi,
    /// Nota fa.
    Fa,
    /// Nota sol.
    Sol,
    /// Nota la.
    La,
    /// Nota si.
    Si,
    /// Nota pause.
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
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'A' | 'a' => Some(Note::La),
            'B' | 'b' => Some(Note::Si),
            'C' | 'c' => Some(Note::Do),
            'D' | 'd' => Some(Note::Re),
            'E' | 'e' => Some(Note::Mi),
            'F' | 'f' => Some(Note::Fa),
            'G' | 'g' => Some(Note::Sol),
            ' ' => Some(Note::Pause),
            _ => None,
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Note::Do
    }
}

/// Estrutura que guarda o estado atual da música.
#[derive(Clone, Copy)]
pub struct State {
    /// BPM
    pub bpm: u8,
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

    pub const MAX_VOLUME: u16 = 16383;

    pub const DEFAULT_BPM: u8 = 80;

    pub const MAX_BPM: u8 = 255;

    /// Cria um estado novo.
    pub fn new(instrument: u8, octave: u8, volume: u16, bpm: u8, note: Note) -> Self {
        Self {
            instrument,
            octave,
            volume,
            bpm,
            note: Some(note),
        }
    }

    pub fn get_tempo(&self) -> u24 {
        bpm_into_micros(self.bpm)
    }
}

pub fn bpm_into_micros(bpm: u8) -> u24 {
    pub const MICROSECS_IN_MINUTE: u32 = 60_000_000;
    u24::from_int_lossy(MICROSECS_IN_MINUTE / bpm as u32)
}

impl Default for State {
    fn default() -> Self {
        Self {
            instrument: 0,
            octave: State::DEFAULT_OCTAVE,
            volume: State::DEFAULT_VOLUME,
            bpm: State::DEFAULT_BPM,
            note: Default::default(),
        }
    }
}

/// Segura informações sobre a música e oferece métodos para seu processamento.
pub struct Sheet {
    /// O BPM da partitura.
    bpm: u8,
    /// O estado atual.
    current_state: State,
    /// Os estados já processados.
    states: Vec<State>,
    /// O texto a ser processado.
    text: String,
}

impl Sheet {
    const DEFAULT_R_PLUS: char = '東';
    const DEFAULT_R_MINUS: char = '世';
    const DEFAULT_BPM_PLUS: char = 'ß';

    /// Cria uma nova partitura a partir de uma BPM básica e um texto.
    pub fn new(bpm: u8, text: impl ToString) -> Self {
        Self {
            bpm,
            states: Vec::new(),
            text: text.to_string(),
            current_state: Default::default(),
        }
    }

    /// Pegar o vetor com os estados e aplicar as mudanças conforme a especificação
    pub fn process(mut self) -> Vec<MIDIaction> {
        let mut ret = Vec::<MIDIaction>::new();

        self.current_state = self.states[0];
        ret.push(MIDIaction::ChangeBPM(self.current_state.bpm));
        ret.push(MIDIaction::ChangeInstrument(self.current_state.instrument));
        ret.push(MIDIaction::ChangeVolume(self.current_state.volume));

        for actual_state in self.states {
            if actual_state.bpm != self.current_state.bpm {
                ret.push(MIDIaction::ChangeBPM(actual_state.bpm));
            } else if actual_state.instrument != self.current_state.instrument {
                ret.push(MIDIaction::ChangeInstrument(actual_state.instrument));
            } else if actual_state.volume != self.current_state.volume {
                ret.push(MIDIaction::ChangeVolume(actual_state.volume));
            } else {
                match actual_state.note {
                    Some(note) => match note {
                        Note::Pause => {
                            ret.push(MIDIaction::Pause(actual_state.bpm as u32));
                        }
                        _ => {
                            ret.push(MIDIaction::PlayNote {
                                bpm: actual_state.bpm as u32,
                                note: (note as u8) + 12 * (actual_state.octave + 1),
                            });
                        }
                    },
                    None => (),
                }
            }
            self.current_state = actual_state;
        }

        ret.push(MIDIaction::EndTrack);
        
        ret
    }
  
    pub fn map_substring_to_char(&mut self) -> String {
        let mut text = self
      
            .text
            .replace("BPM+", &Sheet::DEFAULT_BPM_PLUS.to_string())
            .replace("R+", &Sheet::DEFAULT_R_PLUS.to_string())
            .replace("R-", &Sheet::DEFAULT_R_MINUS.to_string());

        let mut aux = "".to_string();
        let mut prev_char = '\0';

        for c in text.chars() {
            if let Some(new_note) = Note::from_char(prev_char) {
                if matches!(c, 'o' | 'O' | 'I' | 'i' | 'u' | 'U') {
                    aux.push(prev_char);
                    prev_char = c;
                    continue;
                }
            }
            aux.push(c);
            prev_char = c;
        }

        return aux;
    }

    pub fn proccess_text(&mut self) {
        let text = self.map_substring_to_char();

        for c in text.chars() {
            self.parse_char(c);
        }
    }

    /// Altera o current_state e coloca no fim do vetor
    fn parse_char(&mut self, ch: char) {
        print!("{}", ch);

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
                let aux_state = self.states.last().unwrap().clone();
                self.states.push(self.current_state);
                self.current_state = aux_state;
            }

                //R+
                Sheet::DEFAULT_R_PLUS => {
                    // Aumenta UMA oitava; Se não puder, aumentar, volta à oitava default (de início)
                    let new_octave = self.current_state.octave + 1;
                    self.current_state.octave = if new_octave > State::MAX_OCTAVE {
                        State::DEFAULT_OCTAVE
                    } else {
                        new_octave
                    };
                }

                //R-
                Sheet::DEFAULT_R_MINUS => {
                    // Diminui UMA oitava;
                    self.current_state.octave = self.current_state.octave.saturating_sub(1);
                }
                //BPM+
                Sheet::DEFAULT_BPM_PLUS => {
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

        let expected_text = "ß東世".to_string();

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
