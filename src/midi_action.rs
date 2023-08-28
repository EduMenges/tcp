use crate::time_state::TimeState;

use midly::{num::*, *};

/// Enum representando as possíveis ações de MIDI.
#[derive(Clone, Copy)]
pub enum MidiAction {
    /// Toca uma nota semimínima.
    ///
    /// O parâmetro é uma nota do MIDI, ou seja, já ajustada com sua oitava,
    /// onde C4 seria (4 (oitava) + 1 (porque C0 é a nota 12)) * 12 (notas totais, contando acidentes).
    PlayNote(u8),
    /// Muda para um dos 128 instrumentos do General MIDI
    ChangeInstrument(u8),
    /// Muda para um volume contido no intervalo [0, 2^15]
    ChangeVolume(u16),
    /// Pausa por uma semimínima
    Pause,
    /// Troca MSPQN para a BPM dada
    ChangeBPM(u16),
}

impl MidiAction {
    /// Canal padrão
    const D_CHANNEL: u4 = u4::from_int_lossy(0);

    /// Velocidade (força das teclas) padrão
    const D_VELOCITY: u7 = u7::from_int_lossy((i8::MAX / 2) as u8);

    /// Delta para eventos instantâneos
    const INSTANT: u28 = u28::from_int_lossy(0);

    /// Ticks por semimínimas padrão.
    ///
    /// Este valor foi escolhido para maximizar a resolução e qualidade do arquivo.
    pub const D_TPQN: u15 = u15::from_int_lossy(480);

    /// O compasso padrão é 4/4. Igual para todos os arquivos.
    const D_TIME_SIGNATURE: MetaMessage<'_> = midly::MetaMessage::TimeSignature(4, 2, 24, 8);

    /// A escala padrão é C maior. Igual para todos os arquivos
    const D_KEY_SIGNATURE: MetaMessage<'_> = midly::MetaMessage::KeySignature(0, false);

    /// Mensagens a se adicionar no começo de cada trilha. Usado no `to_track`.
    const TO_BE_ADDED: [MetaMessage<'_>; 4] = [
        MetaMessage::TrackName(b"tcp_out"),
        Self::D_TIME_SIGNATURE,
        Self::D_KEY_SIGNATURE,
        MetaMessage::MidiPort(u7::from_int_lossy(0)),
    ];

    /// Transofrma uma sequência de ações em uma trilha válida do MIDI, adicionando
    /// todo o boiler-plate necessário para sua correta reprodução.
    pub fn as_track<'a>(slice: &[Self]) -> Smf<'a> {
        let header: Header = Header {
            format: midly::Format::SingleTrack,
            timing: midly::Timing::Metrical(u15::from_int_lossy(Self::D_TPQN.as_int())),
        };
        let mut smf = Smf::new(header);

        let mut track = Track::new();

        // Add the default meta messages
        Self::add_beggining(&mut track);

        // Main loop
        for action in slice {
            action.push_as_event(&mut track);
        }

        // Finishes
        Self::add_end(&mut track);

        smf.tracks.push(track);
        smf
    }

    /// Adiciona as mensagens iniciais a uma trilha
    fn add_beggining(track: &mut Track) {
        for message in Self::TO_BE_ADDED {
            track.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Meta(message),
            });
        }
    }

    /// Finaliza a trilha.
    fn add_end(track: &mut Track) {
        track.push(TrackEvent {
            delta: u28::from_int_lossy(1),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
    }

    /// Calcula o intervalo de tempo (em ticks) necessário para tocar uma semimínima.
    ///
    /// Como essa quantidade é fixa pelo TPQN do cabeçalho, simplesmente converte
    /// esse valor para u8.
    fn quarter_note_delta() -> u28 {
        u28::from_int_lossy(Self::D_TPQN.as_int() as u32)
    }

    /// Adicioa o a ação como um evento do MIDI para a track passada.
    pub fn push_as_event(self, track: &mut Track) {
        match self {
            Self::PlayNote(note) => {
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
                    delta: Self::quarter_note_delta(),
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::NoteOff {
                            key: note.into(),
                            vel: Self::D_VELOCITY,
                        },
                    },
                });
            }
            Self::ChangeInstrument(instrument) => {
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
            Self::ChangeVolume(volume) => track.push(TrackEvent {
                delta: Self::INSTANT,
                kind: TrackEventKind::Midi {
                    channel: Self::D_CHANNEL,
                    message: MidiMessage::Controller {
                        controller: u7::from_int_lossy(midi_msg::ControlNumber::Volume as u8),
                        value: u7::from_int_lossy(volume as u8),
                    },
                },
            }),
            Self::Pause => {
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
                    delta: Self::quarter_note_delta(),
                    kind: TrackEventKind::Midi {
                        channel: Self::D_CHANNEL,
                        message: MidiMessage::Controller {
                            controller: u7::from_int_lossy(0x7B),
                            value: u7::from_int_lossy(0),
                        },
                    },
                });
            }
            Self::ChangeBPM(bpm) => {
                track.push(TrackEvent {
                    delta: Self::INSTANT,
                    kind: midly::TrackEventKind::Meta(MetaMessage::Tempo(
                        TimeState::mspqn_from_bpm(bpm, 4),
                    )),
                });
            }
        };
    }
}

#[cfg(test)]
mod test {

    use midly::{num::*, Track, TrackEventKind};

    use super::MidiAction;

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
        MidiAction::ChangeInstrument(0).push_as_event(&mut midi_vec);

        // Assert
        assert_eq!(correct, midi_vec[0].kind);
    }
}
