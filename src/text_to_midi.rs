/// Enum com as notas possíveis.
#[derive(Clone, Copy)]
pub enum Note {
    /// Representa uma pausa, um silêncio.
    Pause,
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

impl Note {
    /// A partir de um caractere, cria uma nota.
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'A' => Some(Note::La),
            'B' => Some(Note::Si),
            'C' => Some(Note::Do),
            'D' => Some(Note::Re),
            'E' => Some(Note::Mi),
            'F' => Some(Note::Fa),
            'G' => Some(Note::Sol),
            _ => None,
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Note::Do
    }
}

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MIDIaction {
    Note(Note),
    ChangeInstrument(u8),
    Pause,
    IncreaseVolume,
    IncreaseOctave,
}

/// Estrutura que guarda o estado atual da música.
#[derive(Clone, Copy)]
struct State {
    /// O instrumento atual.
    instrument: u8,
    /// A oitava atual.
    octave: u8,
    /// O volume atual.
    volume: u8,
    /// A nota atual.
    note: Note,
}

impl State {
    /// A máxima oitava possível.
    pub const MAX_OCTAVE: u8 = 12;
    /// A oitava padrão.
    pub const DEFAULT_OCTAVE: u8 = 4;
    /// O volume padrão.
    pub const DEFAULT_VOLUME: u8 = 100;

    /// Cria um estado novo.
    pub fn new(instrument: u8, octave: u8, volume: u8, note: Note) -> Self {
        Self {
            instrument,
            octave,
            volume,
            note,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            instrument: 1,
            octave: State::DEFAULT_OCTAVE,
            volume: State::DEFAULT_VOLUME,
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
        todo!()
    }

    /// Altera o current_state e coloca no fim do vetor
    fn parse_char(&mut self, ch: char) {
        // ABCDEFG
        let new_note: Option<Note> = Note::from_char(ch);
        if let Some(note) = new_note {
            self.current_state.note = note;
            return;
        }

        match ch {
            ' ' => {
                // Aumenta volume para o DOBRO do volume; Se não puder aumentar, volta ao volume default (de início)
                self.current_state.volume = match self.current_state.volume.checked_mul(2) {
                    Some(volume) => volume,
                    None => State::DEFAULT_VOLUME,
                };
                self.current_state.note = Note::Pause;
            }
            '0'..='9' => {
                // Trocar instrumento para o instrumento General MIDI cujo numero é igual ao valor do instrumento ATUAL + valor do dígito
                if let Some(n) = ch.to_digit(10) {
                    self.current_state.instrument += n as u8;
                }
                self.current_state.note = Note::Pause;
            }
            '.' | '?' => {
                // Aumenta UMA oitava; Se não puder, aumentar, volta à oitava default (de início)
                let new_octave = self.current_state.octave + 1;
                self.current_state.octave = if new_octave > State::MAX_OCTAVE {
                    State::DEFAULT_OCTAVE
                } else {
                    new_octave
                };
                self.current_state.note = Note::Pause;
            }
            '!' => {
                // Trocar instrumento para o instrumento General MIDI #114 (Agogo)
                self.current_state.instrument = 114;
                self.current_state.note = Note::Pause;
            }
            '\n' => {
                // Trocar instrumento para o instrumento General MIDI #15 (Tubular Bells)
                self.current_state.instrument = 15;
                self.current_state.note = Note::Pause;
            }
            ';' => {
                // Trocar instrumento para o instrumento General MIDI #76 (Pan Flute)
                self.current_state.instrument = 76;
                self.current_state.note = Note::Pause;
            }
            ',' => {
                // Trocar instrumento para o instrumento General MIDI #20 (Church Organ)
                self.current_state.instrument = 20;
                self.current_state.note = Note::Pause;
            }
            _ => {
                todo!();
            }
        }

        self.states.push(self.current_state);
    }
}
