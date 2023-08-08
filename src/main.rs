mod text_to_midi;
mod midi_action;
mod play;
extern crate midir;

use text_to_midi::Sheet;

fn main() {
    let text = "ABPM+ER+AooEeiR-E;? iBPM+aeioUUGr+R-R-R-R-R-R-R-R-R-R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+R+".to_string();
    
    let mut sheet = Sheet::new(100, text);
    sheet.proccess_text();
}
