use crate::text_to_midi::*;
use midly::{num::*, *};

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MIDIaction {
    PlayNote { bpm: u8, note: u8 },
    ChangeInstrument(u8),
    ChangeVolume(u16),
    ChangeOctave(u8),
    ChangeBPM(u16),
    Pause,
}

impl MIDIaction {
    const DEFAULT_TRACK: usize = 0;
    const DEFAULT_CHANNEL: u4 = 0.into();
    const DEFAULT_VELOCITY: u7 = (127 / 2).into();

    pub fn to_events(self, vec: &mut Track) {
        match self {
            MIDIaction::PlayNote { bpm, note } => {
                vec.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::NoteOn {
                            key: note.into(),
                            vel: Self::DEFAULT_VELOCITY,
                        },
                    },
                });
            }
            MIDIaction::ChangeInstrument(_) => todo!(),
            MIDIaction::ChangeVolume(_) => todo!(),
            MIDIaction::ChangeOctave(_) => todo!(),
            MIDIaction::ChangeBPM(_) => todo!(),
            MIDIaction::Pause => todo!(),
        };
    }
}
