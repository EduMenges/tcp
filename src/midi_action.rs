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
    const D_CHANNEL: u4 = u4::from_int_lossy(0);
    const D_VELOCITY: u7 = u7::from_int_lossy(127 / 2);
    /// Instant delta
    const INSTANT: u28 = u28::from_int_lossy(0);
    /// Pulses Per Quarter Note
    pub const D_PPQN: u16 = 480;
    /// Is 4/4
    const D_TIME_SIGNATURE: midly::MetaMessage<'_> = midly::MetaMessage::TimeSignature(4, 2, 24, 8);
    /// Is C major
    const D_KEY_SIGNATURE: midly::MetaMessage<'_> = midly::MetaMessage::KeySignature(0, false);
    const DEFAULT_MIDI_PORT: midly::MetaMessage<'_> =
        midly::MetaMessage::MidiPort(u7::from_int_lossy(0));
    const TO_BE_ADDED: [MetaMessage<'_>; 3] = [
        Self::D_TIME_SIGNATURE,
        Self::D_KEY_SIGNATURE,
        Self::DEFAULT_MIDI_PORT,
    ];

    pub fn to_track<'a>(slice: &[Self]) -> Smf<'a> {
        let header: Header = Header {
            format: midly::Format::SingleTrack,
            timing: midly::Timing::Metrical(u15::from_int_lossy(Self::D_PPQN)),
        };
        let mut smf = Smf::new(header);

        let mut track = Track::new();

        // Add the default meta messages
        Self::add_beggining(&mut track);

        // Main loop
        for action in slice {
            action.push_as_event(&mut track)
        }

        smf.tracks.push(track);
        smf
    }

    fn bpm_into_ticks(bpm: u32) -> u28 {
        u28::from_int_lossy(60_000_000 / (bpm * Self::D_PPQN as u32))
    }

    fn add_beggining(track: &mut Track) {
        for message in Self::TO_BE_ADDED {
            track.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Meta(message),
            });
        }
    }

    pub fn push_as_event(self, track: &mut Track) {
        match self {
            MIDIaction::PlayNote { bpm, note } => {
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::NoteOn {
                            key: note.into(),
                            vel: Self::D_VELOCITY,
                        },
                    },
                });
                track.push(TrackEvent {
                    delta: Self::bpm_into_ticks(bpm),
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::NoteOff {
                            key: note.into(),
                            vel: Self::D_VELOCITY,
                        },
                    },
                });
            }
            MIDIaction::ChangeInstrument(instrument) => {
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::ProgramChange {
                            program: u7::from_int_lossy(instrument),
                        },
                    },
                });
            }
            MIDIaction::ChangeVolume(volume) => track.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Midi {
                    channel: Self::D_CHANNEL,
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
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
                track.push(TrackEvent {
                    delta: Self::bpm_into_ticks(bpm),
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
            }
            MIDIaction::ChangeBPM(bpm) => {
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: midly::TrackEventKind::Meta(MetaMessage::Tempo(bpm_into_micros(bpm))),
                });
            }
            MIDIaction::EndTrack => track.push(TrackEvent {
                delta: Self::INSTANT,
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
        MIDIaction::ChangeInstrument(0).push_as_event(&mut midi_vec);

        // Assert
        assert_eq!(correct, midi_vec[0].kind);
    }
}
