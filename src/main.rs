#![allow(unused)]

mod midi_action;
mod play;
mod text_to_midi;
mod time_state;
pub mod user_interface;
mod note;
extern crate midir;

use text_to_midi::Sheet;
use user_interface::UserInterface;
fn main() {
    eframe::run_native(
        "Text to MIDI",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(UserInterface::default())),
    );
}
