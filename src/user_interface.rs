use eframe::{
    egui::{CentralPanel, Context, TextEdit},
    App, Frame,
};
use egui_file::FileDialog;
use midly::Smf;
use std::{fs, path::Path, path::PathBuf};

use crate::{
    midi_action::MidiAction,
    play::{self, play_file},
    text_to_midi::{self, State},
};

#[derive(Default)]
pub struct UserInterface {
    opened_file: Option<PathBuf>,
    saved_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    saved_file_dialog: Option<FileDialog>,
    file_content: String,
    bpm: u16,
    volume: u16,
}

impl UserInterface {
    pub fn new_interface() -> Self {
        UserInterface {
            opened_file: None,
            saved_file: None,
            open_file_dialog: None,
            saved_file_dialog: None,
            file_content: String::new(),
            bpm: State::D_BPM,
            volume: State::D_VOLUME,
        }
    }
}

impl App for UserInterface {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                if (ui.button("Open")).clicked() {
                    let mut dialog = FileDialog::open_file(self.opened_file.clone());
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }

                if (ui.button("Play")).clicked() {
                    let test = text_to_midi::Sheet::new(
                        self.bpm,
                        self.volume,
                        self.file_content.to_string(),
                    );
                    let actions = test.process();
                    let file = MidiAction::as_track(&actions);
                    let _ = play_file(&file);
                }

                if (ui.button("Save")).clicked() {
                    let mut dialog = FileDialog::save_file(self.saved_file.clone());
                    dialog.open();
                    self.saved_file_dialog = Some(dialog);
                }

                ui.add(egui::Slider::new(&mut self.bpm, 0..=State::MAX_BPM).text("BPM"));

                ui.add(egui::Slider::new(&mut self.volume, 0..=State::MAX_VOLUME).text("Volume"));

                if let Some(dialog) = &mut self.open_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(file) = dialog.path() {
                            self.opened_file = Some(file.to_path_buf());
                            // Read file content and store it
                            if let Ok(content) = fs::read_to_string(&file) {
                                self.file_content = content;
                            }
                        }
                    }
                }

                if let Some(dialog) = &mut self.saved_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(file) = dialog.path() {
                            self.saved_file = Some(file.to_path_buf());
                            let test = text_to_midi::Sheet::new(
                                self.bpm,
                                self.volume,
                                self.file_content.to_string(),
                            );
                            let actions = test.process();
                            let midi_file = MidiAction::as_track(&actions);

                            if let Some(saved_file) = &self.saved_file {
                                let mut saved_file = saved_file.clone();
                                saved_file.set_extension("mid");
                                let _ = midi_file.save(saved_file);
                            }
                        }
                    }
                }
            });

            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        //ui.label("File Content:");
                        ui.text_edit_multiline(&mut self.file_content);
                    });
                });
        });
    }
}
