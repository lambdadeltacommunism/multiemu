use crate::{config::GlobalConfig, env::STORAGE_DIRECTORY};
use egui::{CentralPanel, Context, ScrollArea, SidePanel};
use itertools::Itertools;
use std::{
    fs::read_dir,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(PartialEq, Eq, Clone, Debug)]
struct FileBrowserState {
    pub path: PathBuf,
    pub directory_contents: Vec<PathBuf>,
}

impl FileBrowserState {
    pub fn new() -> Self {
        let mut me = Self {
            path: PathBuf::default(),
            directory_contents: Vec::default(),
        };

        me.change_directory(STORAGE_DIRECTORY.deref());

        me
    }

    pub fn change_directory(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.path = path.clone();
        self.directory_contents = read_dir(path)
            .unwrap()
            .map(|x| x.unwrap().path())
            .sorted()
            .collect();
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum MenuItem {
    #[default]
    Main,
    FileBrowser,
    Options,
    Database,
}

#[derive(Clone, Debug)]
pub struct GuiRuntime {
    pub active: bool,
    open_menu_item: MenuItem,
    file_browser_state: FileBrowserState,
    global_config: Arc<RwLock<GlobalConfig>>,
}

impl GuiRuntime {
    pub fn new(global_config: Arc<RwLock<GlobalConfig>>) -> Self {
        Self {
            active: true,
            open_menu_item: MenuItem::Main,
            file_browser_state: FileBrowserState::new(),
            global_config,
        }
    }

    /// TODO: barely does anything
    pub fn main_menu_logic(&mut self, ctx: &Context) {
        SidePanel::left("options_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Main").clicked() {
                            self.open_menu_item = MenuItem::Main;
                        }

                        if ui.button("File Browser").clicked() {
                            self.open_menu_item = MenuItem::FileBrowser;
                        }

                        if ui.button("Options").clicked() {
                            self.open_menu_item = MenuItem::Options;
                        }

                        if ui.button("Database").clicked() {
                            self.open_menu_item = MenuItem::Database;
                        }
                    })
                })
            });

        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::LEFT),
                    |ui| match self.open_menu_item {
                        MenuItem::Main => if ui.button("Resume").clicked() {},
                        MenuItem::FileBrowser => {
                            let mut new_dir = None;

                            ui.horizontal(|ui| {
                                for (index, path_segment) in
                                    self.file_browser_state.path.iter().enumerate()
                                {
                                    if index != 0 {
                                        ui.label("/");
                                    }

                                    if ui.button(path_segment.to_str().unwrap()).clicked() {
                                        new_dir = Some(PathBuf::from_iter(
                                            self.file_browser_state.path.iter().take(index + 1),
                                        ));
                                    }
                                }
                            });

                            if ui.button("..").clicked() {
                                new_dir = self
                                    .file_browser_state
                                    .path
                                    .parent()
                                    .map(|p| p.to_path_buf());
                            }

                            for file_entry in self.file_browser_state.directory_contents.iter() {
                                let file_name = file_entry.file_name().unwrap().to_str().unwrap();

                                if ui.button(file_name).clicked() && file_entry.is_dir() {
                                    new_dir = Some(file_entry.clone());
                                }
                            }

                            if let Some(new_dir) = new_dir {
                                tracing::trace!("Changing directory to {:?}", new_dir);
                                self.file_browser_state.change_directory(new_dir);
                            }
                        }
                        MenuItem::Options => {
                            let mut global_config = self.global_config.write().unwrap();

                            ui.horizontal(|ui| {
                                ui.checkbox(
                                    &mut global_config.hardware_acceleration,
                                    "Hardware Acceleration",
                                );
                            });
                        }
                        MenuItem::Database => {}
                    },
                );
            });
        });
    }
}
