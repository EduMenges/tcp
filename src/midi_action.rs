use crate::text_to_midi::*;
use midi_msg::MidiMsg;
use midly::{num::*, *};

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MIDIaction {
    PlayNote { bpm: u32, note: u8 },
    ChangeInstrument(u8),
    ChangeVolume(u16),
    Pause(u32),
    ChangeBPM(u8),
    EndTrack,
}

impl MIDIaction {
    const DEFAULT_CHANNEL: u4 = u4::from_int_lossy(0);
    const DEFAULT_VELOCITY: u7 = u7::from_int_lossy(127 / 2);
    /// Instant delta
    const INSTANT: u28 = u28::from_int_lossy(0);
    /// Pulses Per Quarter Note
    const DEFAULT_PPQN: u16 = 480;

    pub fn to_track<'a>(slice: &[Self]) -> Smf<'a> {
        let header: Header = Header {
            format: midly::Format::SingleTrack,
            timing: midly::Timing::Metrical(u15::from_int_lossy(Self::DEFAULT_PPQN)),
        };
        let mut smf = Smf::new(header);

        let mut track = Track::new();

        for action in slice {
            action.push_as_event(&mut track)
        }

        smf.tracks.push(track);
        smf
    }

    pub fn to_events(self, track: &mut Track) {
        match self {
            MIDIaction::PlayNote { bpm, note } => {
                track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::NoteOn {
                            key: note.into(),
                            vel: Self::DEFAULT_VELOCITY,
                        },
                    },
                });
                track.push(TrackEvent {
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
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::ProgramChange {
                            program: u7::from_int_lossy(instrument),
                        },
                    },
                });
            }
            MIDIaction::ChangeVolume(volume) => track.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Midi {
                    channel: Self::DEFAULT_CHANNEL,
                    message: MidiMessage::Controller {
                        controller: u7::from_int_lossy(midi_msg::ControlNumber::Volume as u8),
                        value: u7::from_int_lossy(volume as u8),
                    },
                },
            }),
            MIDIaction::Pause(bpm) => {
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::DEFAULT_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
                track.push(TrackEvent {
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
            MIDIaction::ChangeBPM(bpm) => {
                track.push(TrackEvent {
                    delta: u28::default(),
                    kind: midly::TrackEventKind::Meta(MetaMessage::Tempo(bpm_into_micros(bpm))),
                });
            }
            MIDIaction::EndTrack => track.push(TrackEvent {
                delta: u28::default(),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            }),
        };
    }
}

#[cfg(test)]
mod test {

    use midly::{num::*, Track, TrackEventKind};

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
