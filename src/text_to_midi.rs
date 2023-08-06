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

    pub const MAX_BPM: u8 = 3200;

    pub const MICROSECS_IN_MINUTE: u32 = 60_000_000;

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
    u24::from_int_lossy(State::MICROSECS_IN_MINUTE / bpm as u32)
}

impl Default for State {
    fn default() -> Self {
        Self {
            instrument: 1,
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
    /// Cria uma nova partitura a partir de uma BPM básica e um texto.
    pub fn new(bpm: u8, text: String) -> Self {
        Self {
            bpm,
            states: Vec::new(),
            text,
            current_state: Default::default(),
        }
    }

    /// Pegar o vetor com os estados e aplicar as mudanças conforme a especificação
    pub fn proccess(self) -> Vec<MIDIaction> {
        let mut ret = Vec::<MIDIaction>::new();
        ret.push(MIDIaction::EndTrack);
        todo!();
        return ret;
    }

    pub fn into_bytes<'a>(actions: Vec<MIDIaction>) -> Smf<'a> {
        let header: Header = Header {
            format: midly::Format::SingleTrack,
            timing: midly::Timing::Metrical(u15::from_int_lossy(480)),
        };
        let mut smf = Smf::new(header);

        let mut track = Track::new();

        for action in actions {
            action.to_events(&mut track)
        }

        smf.tracks.push(track);
        smf
    }

    /// Altera o current_state e coloca no fim do vetor
    fn parse_char(&mut self, ch: char) {
        //let ch = self.text.chars().nth(index).unwrap();

        // ABCDEFG
        let new_note: Option<Note> = Note::from_char(ch);
        if let Some(note) = new_note {
            self.current_state.note = Some(note);
            return;
        } else {
            self.current_state.note = None;
        }

        //R+


        match ch {
            '+' => {
                // Aumenta volume para o DOBRO do volume; Se não puder aumentar, volta ao volume default (de início)
                self.current_state.volume = if self.current_state.volume * 2 > State::MAX_VOLUME {
                    State::MAX_VOLUME
                } else {
                    self.current_state.volume * 2
                };
            }
            '-' => {
                // Volume retorna ao volume padrão
                self.current_state.volume = State::DEFAULT_VOLUME;
            }

            'o'|'O'|'I'|'i'|'u'|'U' => {
                // Nesse caso, caso que em que não há uma nota anterior, altera o instrumento para o telefone
                self.current_state.instrument = 125;
            }
            //R+
            '東' => {
                // Aumenta UMA oitava; Se não puder, aumentar, volta à oitava default (de início)
                let new_octave = self.current_state.octave + 1;
                self.current_state.octave = if new_octave > State::MAX_OCTAVE {
                    State::DEFAULT_OCTAVE
                } else {
                    new_octave
                };
            }

             //R-
             '京' => {
                // Diminui UMA oitava; Se não puder, diminuir, volta à oitava default (de início)
                let new_octave = self.current_state.octave - 1;
                self.current_state.octave = if new_octave < 0 {
                    State::DEFAULT_OCTAVE
                } else {
                    new_octave
                };
            }
            //BPM+
            'ß' => {
                // Aumenta BPM em 80 unidades
                self.current_state.bpm += 80;
                self.current_state.bpm = if self.current_state.bpm + 80> State::MAX_BPM {
                    State::MAX_BPM
                } else {
                    self.current_state.bpm + 80
                };
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
                self.current_state.instrument = rng.gen_range(0..128);
            }

            ';' => {
                //Atribui valor aleatorio ao BPM
                let mut rng = rand::thread_rng();
                self.current_state.bpm = rng.gen_range(1..State::MAX_BPM);
            }
            _ => {
                todo!();
            }
        }

        self.states.push(self.current_state);
    }
}
