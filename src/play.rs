use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use midi_msg::{MidiMsg, ReceiverContext};
use midir::{MidiOutput, MidiOutputPort};
use midly::num::*;
use midly::{Smf, Track};

fn ticks_to_micros(ticks: u28, mspqn: u24, ppqn: u15) -> u64 {
    (ticks.as_int() as u64 * mspqn.as_int() as u64) / ppqn.as_int() as u64
}

pub fn play_file<'a>(file: &Smf<'a>) -> Result<(), Box<dyn Error>> {
    let midi_out = MidiOutput::new("Output")?;

    // Get an output port (read from console if multiple are available)
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

    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(out_port, "midir-test")?;
    println!("Connection open. Listen!");

    let mut buf = Vec::new();
    let ppqn = match file.header.timing {
        midly::Timing::Metrical(as_u15) => as_u15,
        midly::Timing::Timecode(_, _) => panic!("Only headers with Metrical coding are necessary"),
    };

    let mut current_mspqn: u24;

    for event in file.tracks[0] {
        if event.delta > 0 {
            sleep(Duration::from_micros(ticks_to_micros(
                event.delta,
                current_mspqn,
                ppqn,
            )))
        }
        match event.kind.as_live_event() {
            Some(event) => {
                event.write_std(buf);
                conn_out.send(&buf);
            }
            None => {
                if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(mspqn)) = event.kind {
                    current_mspqn = mspqn;
                }
            }
        }
    }

    sleep(Duration::from_millis(150));
    println!("\nClosing connection");
    // This is optional, the connection would automatically be closed as soon as it goes out of scope
    conn_out.close();
    println!("Connection closed");
    Ok(())
}