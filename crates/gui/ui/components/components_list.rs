use std::path::PathBuf;
use iced::widget::{button, column, container, horizontal_space, progress_bar, row, text, toggler};
use iced::{alignment, Alignment, Command, Element, Length, Theme};
use anime_launcher_sdk::anime_game_core::prelude::*;
use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::honkai::config::Config;

use crate::models::{ComponentGroup, ComponentVersion, ComponentsListPattern, DownloadStatus};
use crate::helpers::prettify_bytes;

#[derive(Debug, Clone)]
pub enum ComponentMsg {
    ToggleShowRecommended(bool),
    ToggleGroupExpand(usize),
    InstallVersion { group_idx: usize, ver_idx: usize },
    DeleteVersion { group_idx: usize, ver_idx: usize },
    DownloadProgress {
        group_idx: usize,
        ver_idx: usize,
        fraction: f32,
        caption: String,
    },
    DownloadFinished { group_idx: usize, ver_idx: usize },
    DownloadFailed { group_idx: usize, ver_idx: usize, error: String },
}

pub struct ComponentsListView {
    pub download_folder: PathBuf,
    pub groups: Vec<ComponentGroup>,
    pub show_recommended_only: bool,
}

impl ComponentsListView {
    pub fn new(pattern: ComponentsListPattern) -> Self {
        let mut slf = Self {
            download_folder: pattern.download_folder,
            groups: pattern.groups,
            show_recommended_only: true,
        };
        slf.refresh_installed_states();
        slf
    }

    pub fn refresh_installed_states(&mut self) {
        for group in &mut self.groups {
            for ver in &mut group.versions {
                let path = self.download_folder.join(&ver.name);
                if path.exists() {
                    ver.status = DownloadStatus::Completed;
                } else if !matches!(ver.status, DownloadStatus::Downloading { .. }) {
                    ver.status = DownloadStatus::Idle;
                }
            }
        }
    }

    pub fn update(&mut self, message: ComponentMsg) -> Command<ComponentMsg> {
        match message {
            ComponentMsg::ToggleShowRecommended(show) => {
                self.show_recommended_only = show;
                Command::none()
            }

            ComponentMsg::ToggleGroupExpand(group_idx) => {
                if let Some(group) = self.groups.get_mut(group_idx) {
                    group.is_expanded = !group.is_expanded;
                }
                Command::none()
            }

            ComponentMsg::DeleteVersion { group_idx, ver_idx } => {
                if let Some(group) = self.groups.get_mut(group_idx) {
                    if let Some(ver) = group.versions.get_mut(ver_idx) {
                        let path = self.download_folder.join(&ver.name);
                        if path.exists() {
                            let _ = std::fs::remove_dir_all(path);
                        }
                        ver.status = DownloadStatus::Idle;
                    }
                }
                Command::none()
            }

            ComponentMsg::InstallVersion { group_idx, ver_idx } => {
                let Some(group) = self.groups.get(group_idx) else { return Command::none(); };
                let Some(ver) = group.versions.get(ver_idx) else { return Command::none(); };

                let uri = ver.uri.clone();
                let download_folder = self.download_folder.clone();
                let filename = ver.format.as_ref().map(|f| format!("{}.{f}", ver.name));

                if let Some(v) = self.groups.get_mut(group_idx).and_then(|g| g.versions.get_mut(ver_idx)) {
                    v.status = DownloadStatus::Downloading {
                        fraction: 0.0,
                        caption: "Starting...".into(),
                    };
                }

                Command::perform(
                    async move {
                        let (sender, mut receiver) = iced::futures::channel::mpsc::unbounded();
                        
                        std::thread::spawn(move || {
                            let temp_dir = Config::get()
                                .ok()
                                .and_then(|c| c.launcher.temp)
                                .unwrap_or_else(std::env::temp_dir);

                            let Ok(mut installer) = Installer::new(&uri) else {
                                let _ = sender.unbounded_send(Err("Failed to init installer".into()));
                                return;
                            };

                            installer = installer.with_temp_folder(temp_dir);
                            if let Some(f) = filename {
                                installer = installer.with_filename(f);
                            }

                            let s = sender.clone();
                            installer.install(download_folder, move |update| {
                                match update {
                                    InstallerUpdate::DownloadingProgress(curr, total) => {
                                        let frac = curr as f32 / total as f32;
                                        let cap = format!("Downloading: {} / {}", prettify_bytes(curr), prettify_bytes(total));
                                        let _ = s.unbounded_send(Ok((frac, cap)));
                                    }
                                    InstallerUpdate::UnpackingProgress(curr, total) => {
                                        let frac = curr as f32 / total as f32;
                                        let cap = format!("Unpacking: {frac:.1}%");
                                        let _ = s.unbounded_send(Ok((frac, cap)));
                                    }
                                    InstallerUpdate::UnpackingFinished => {
                                        // Hoàn thành
                                    }
                                    InstallerUpdate::DownloadingError(e) | InstallerUpdate::UnpackingError(e) => {
                                        let _ = s.unbounded_send(Err(format!("{e:?}")));
                                    }
                                    _ => {}
                                }
                            });
                        });

                    },
                    |_| ComponentMsg::DownloadFinished { group_idx, ver_idx }
                )
            }

            ComponentMsg::DownloadProgress { group_idx, ver_idx, fraction, caption } => {
                if let Some(v) = self.groups.get_mut(group_idx).and_then(|g| g.versions.get_mut(ver_idx)) {
                    v.status = DownloadStatus::Downloading { fraction, caption };
                }
                Command::none()
            }

            ComponentMsg::DownloadFinished { group_idx, ver_idx } => {
                if let Some(v) = self.groups.get_mut(group_idx).and_then(|g| g.versions.get_mut(ver_idx)) {
                    v.status = DownloadStatus::Completed;
                }
                Command::none()
            }

            ComponentMsg::DownloadFailed { group_idx, ver_idx, error } => {
                if let Some(v) = self.groups.get_mut(group_idx).and_then(|g| g.versions.get_mut(ver_idx)) {
                    v.status = DownloadStatus::Error(error);
                }
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, ComponentMsg> {
        let mut content = column![
            row![
                text("Components Manager").size(20),
                horizontal_space(),
                toggler(
                    String::from("Show recommended only"),
                    self.show_recommended_only,
                    ComponentMsg::ToggleShowRecommended
                )
            ]
            .padding(10)
            .align_items(Alignment::Center)
        ]
        .spacing(12);

        for (g_idx, group) in self.groups.iter().enumerate() {
            let expand_icon = if group.is_expanded { "▼" } else { "▶" };
            
            // Header của Accordion Group
            let group_header = button(
                row![
                    text(expand_icon).size(12),
                    text(&group.title).size(16),
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(ComponentMsg::ToggleGroupExpand(g_idx))
            .width(Length::Fill)
            .padding(8);

            let mut group_col = column![group_header].spacing(4);

            if group.is_expanded {
                for (v_idx, ver) in group.versions.iter().enumerate() {
                    // Lọc chỉ hiển thị bản recommended nếu bật cờ
                    if self.show_recommended_only && !ver.recommended && ver.status == DownloadStatus::Idle {
                        continue;
                    }

                    group_col = group_col.push(self.view_version_row(g_idx, v_idx, ver));
                }
            }

            content = content.push(
                container(group_col)
                    .padding(6)
                    .style(iced::theme::Container::Box)
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into()
    }

    fn view_version_row<'a>(&'a self, g_idx: usize, v_idx: usize, ver: &'a ComponentVersion) -> Element<'a, ComponentMsg> {
        let title_col = column![
            text(&ver.title).size(14),
            text(if ver.recommended { "Recommended" } else { "Optional" })
                .size(11)
        ]
        .width(Length::FillPortion(3));

        let action_area: Element<'a, ComponentMsg> = match &ver.status {
            DownloadStatus::Idle => {
                button(text("Download").size(12))
                    .on_press(ComponentMsg::InstallVersion { group_idx: g_idx, ver_idx: v_idx })
                    .padding([4, 12])
                    .into()
            }
            DownloadStatus::Downloading { fraction, caption } => {
                column![
                    text(caption).size(11),
                    progress_bar(0.0..=1.0, *fraction).height(8.0)
                ]
                .width(Length::FillPortion(2))
                .spacing(4)
                .into()
            }
            DownloadStatus::Completed => {
                row![
                    text("Installed").size(12),
                    button(text("Delete").size(12))
                        .on_press(ComponentMsg::DeleteVersion { group_idx: g_idx, ver_idx: v_idx })
                        .padding([4, 8])
                        .style(iced::theme::Button::Destructive)
                ]
                .spacing(8)
                .align_items(Alignment::Center)
                .into()
            }
            DownloadStatus::Error(err) => {
                row![
                    text(format!("Error: {err}")).size(11),
                    button(text("Retry").size(12))
                        .on_press(ComponentMsg::InstallVersion { group_idx: g_idx, ver_idx: v_idx })
                ]
                .spacing(8)
                .into()
            }
        };

        row![
            title_col,
            horizontal_space(),
            action_area
        ]
        .padding(6)
        .align_items(Alignment::Center)
        .into()
    }
}
