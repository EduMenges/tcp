use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use crate::time_state::*;
use midi_msg::{MidiMsg, ReceiverContext};
use midir::{MidiOutput, MidiOutputPort};
use midly::{num::*, Fps};
use midly::{Smf, Track};

use crate::midi_action::MIDIaction;

pub fn play_file<'a>(file: &Smf<'a>) -> Result<(), Box<dyn Error>> {
    let mut conn_out = prepare_connection()?;

    let mut buf = Vec::new();
    let mut time_state = TimeState::default();
    let tpqn = match file.header.timing {
        midly::Timing::Metrical(as_u15) => as_u15,
        midly::Timing::Timecode(_, _) => panic!("Only headers with Metrical coding can be parsed"),
    };
    time_state.tpqn = tpqn;

    for event in file.tracks[0].iter() {
        if event.delta > 0 {
            sleep(time_state.duration_per_tick() * event.delta.as_int())
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

fn prepare_connection() -> Result<midir::MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("Output")?;
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return Err("no output port found".into()),
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
    let mut conn_out = midi_out.connect(out_port, "midir")?;
    println!("Connection open");
    Ok(conn_out)
}

#[cfg(test)]
mod test {
    use crate::{midi_action::MIDIaction, text_to_midi};

    use super::*;

    #[test]
    fn from_empty_midi() {
        let smf = Smf::parse(include_bytes!("../test-asset/empty.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn from_regular_file() {
        let smf = Smf::parse(include_bytes!("../test-asset/twinkle.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn scale_from_ours() {
        play("CDEFGABR+C");
    }

    #[test]
    fn scale_from_mocked_file() {
        let smf = Smf::parse(include_bytes!("../test-asset/c_major_scale_200_bpm.mid")).unwrap();

        let _ = play_file(&smf);
    }

    #[test]
    fn scale_200_bpm() {
        play("BPM+CDEFGABR+C");
    }


    fn play(text: &str) {
        let mut test = text_to_midi::Sheet::new(120, text);
        test.process_text();
        let actions = test.process();

        let _ = play_file(&MIDIaction::to_track(&actions));
    }
    
    #[test]
    fn descending_major_scale() {
        play("EDCR-BAGFEDCR-BAG");
    }

    #[test]
    /// Contains UP in BPM
    fn tubular_bells(){
        play("BPM+BPM+R+EAEBEGAER+CR-ER+DR-EBR+CR-EAEBEGAER+CR-ER+DR-EBR+CR-EB");
    }

    #[test]
    fn pause() {
        play("C D E F");
    }

    #[test]
    fn scale_with_varying_cases()
    {
        play("cDeFgAb");
    }

    #[test]
    fn remaining_vowels() {
        play("Ciiou");
    }

    #[test]
    fn random_notes() {
        play("??????");
    }
}
