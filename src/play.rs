use std::error::Error;

use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use crate::time_state::*;

use midir::{MidiOutput, MidiOutputPort};

use midly::Smf;

/// Reproduz o dado arquivo com os sintetizadores disponíveis no sistema.
///
/// Caso o arquivo passado não seja codificado em métrico, retorna erro.
pub fn play_file(file: &Smf<'_>) -> Result<(), Box<dyn Error>> {
    let mut conn_out = prepare_connection()?;

    let mut buf = Vec::new();
    let mut time_state = TimeState::default();
    time_state.tpqn = match file.header.timing {
        midly::Timing::Metrical(as_u15) => as_u15,
        midly::Timing::Timecode(_, _) => {
            return Err("The timing of the received file is not coded with metrical.".into())
        }
    };

    for event in &file.tracks[0] {
        if event.delta > 0 {
            sleep(time_state.duration_per_tick() * event.delta.as_int());
        }
        match event.kind.as_live_event() {
            Some(event) => {
                let _ = event.write(&mut buf);
                let _ = conn_out.send(&buf);
            }
            None => match event.kind {
                midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(mspqn)) => {
                    time_state.set_mspqn(mspqn);
                }
                midly::TrackEventKind::Meta(midly::MetaMessage::TimeSignature(
                    numerator,
                    denominator,
                    _,
                    _,
                )) => {
                    time_state.set_time_signature(TimeSignature::from_raw(numerator, denominator));
                }
                _ => (),
            },
        }
        buf.clear();
    }

    sleep(Duration::from_millis(150));
    println!("\nClosing connection");

    Ok(())
}

/// Abre uma conexão com uma das portas MIDI disponíveis.
fn prepare_connection() -> Result<midir::MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("TCP")?;
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return Err("No output port found.".into()),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0]).unwrap()
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");

            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }

            print!("Please select output port: ");
            stdout().flush()?;

            let mut input = String::new();
            stdin().read_line(&mut input)?;

            out_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("Invalid output port selected.")?
        }
    };
    println!("Opening connection");

    let conn_out = midi_out.connect(out_port, "midir")?;
    println!("Connection open");

    Ok(conn_out)
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::{main, midi_action::MidiAction, text_to_midi};

    use super::*;

    #[test]
    fn from_empty_midi() {
        let smf = Smf::parse(include_bytes!("../test-asset/empty.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn scale_from_ours() {
        play("CDEFGABR+C");
    }

    #[test]
    fn scale_from_mocked_file() {
        let smf = Smf::parse(include_bytes!("../test-asset/c_major_scale.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn scale_200_bpm() {
        let actions = text_to_midi::Sheet::with_default_volume(200, "CDEFGABR+C");
        let file = MidiAction::as_track(&actions.process());
        let _ = play_file(&file);
        let _ = file.save("../200bpm.mid");
    }

    #[test]
    fn twinkle_mocked() {
        let smf = Smf::parse(include_bytes!("../test-asset/twinkle.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn twinkle_ours() {
        let _ = play("CCGGAAG FFEEDDC GGFFEED GGFFEED CCGGAAG FFEEDDC");
    }

    fn play(text: impl ToString) {
        let test = text_to_midi::Sheet::with_default_volume(120, text.to_string());
        let actions = test.process();

        let _ = play_file(&MidiAction::as_track(&actions));
    }

    #[test]
    fn descending_major_scale() {
        play("EDCR-BAGFEDCR-BAG");
    }

    #[test]
    /// Contains UP in BPM, UP and LOW in octave
    fn tubular_bells() {
        let start = "BPM+BPM+R+".to_owned();
        let main_loop = "EAEBEGAER+CR-ER+DR-EBR+CR-EAEBEGAER+CR-ER+DR-EBR+CR-EB";
        let actions = text_to_midi::Sheet::with_default_volume(
            140,
            (0..10).fold(start, |acc, _| acc + main_loop + "\n"),
        )
        .process();
        let file = MidiAction::as_track(&actions);
        let _ = file.save("../tubular_bells.mid");
        let _ = play_file(&file);
    }

    #[test]
    fn pause() {
        play("C D E F");
    }

    #[test]
    fn scale_with_varying_cases() {
        play("cDeFgAb");
    }

    #[test]
    fn major_scale_with_volume() {
        play("C+D+E+F+G+A+B+")
    }

    #[test]
    /// Should play telephone sound
    fn remaining_vowels() {
        play("Ciiou");
    }

    #[test]
    fn random_notes() {
        play("??????");
    }
}
