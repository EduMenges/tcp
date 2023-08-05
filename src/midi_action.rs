use crate::text_to_midi::*;
use midi_msg::MidiMsg;
use midly::{num::*, *};

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MIDIaction {
    PlayNote { bpm: u32, note: u8 },
    ChangeInstrument(u8),
    ChangeVolume(u8),
    Pause(u32),
}

impl MIDIaction {
    const DEFAULT_TRACK: usize = 0;
    const DEFAULT_CHANNEL: u4 = u4::from_int_lossy(0);
    const DEFAULT_VELOCITY: u7 = u7::from_int_lossy(127 / 2);
    const INSTANT: u28 = u28::from_int_lossy(0);

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
                vec.push(TrackEvent {
                    delta: u28::from_int_lossy(bpm),
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::NoteOff {
                            key: note.into(),
                            vel: Self::DEFAULT_VELOCITY,
                        },
                    },
                });
            }
            MIDIaction::ChangeInstrument(instrument) => {
                vec.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::ProgramChange {
                            program: u7::from_int_lossy(instrument),
                        },
                    },
                });
            }
            MIDIaction::ChangeVolume(volume) => vec.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Midi {
                    channel: Self::DEFAULT_CHANNEL,
                    message: MidiMessage::Controller {
                        controller: u7::from_int_lossy(midi_msg::ControlNumber::Volume as u8),
                        value: u7::from_int_lossy(volume),
                    },
                },
            }),
            MIDIaction::Pause(bpm) => {
                vec.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
                vec.push(TrackEvent {
                    delta: u28::from_int_lossy(bpm),
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
            }
        };
    }
}

#[cfg(test)]
mod test {
    use midi_msg::MidiMsg;
    use midly::{Track, TrackEventKind, num::*};

    use super::MIDIaction;

    #[test]
    fn change_instrument() {
        // Arrange
        let correct = TrackEventKind::Midi {
            channel: u4::from_int_lossy(0),
            message: midly::MidiMessage::ProgramChange {
                program: u7::from_int_lossy(0),
            },
        };

        let mut midi_vec = Track::new();
        MIDIaction::ChangeInstrument(0).to_events(&mut midi_vec);

        // Assert
        assert_eq!(correct, midi_vec[0].kind);
    }
}
