use crate::text_to_midi::Note;

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MIDIaction {
    Note(Note),
    ChangeInstrument(u8),
    Pause,
    ChangeVolume(u16),
    ChangeOctave(u8),
    PlayRandom,
    IncreaseBPM,
    RandomBPM,
    Pause,
}