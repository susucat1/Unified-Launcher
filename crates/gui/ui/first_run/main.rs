use anime_launcher_sdk::components::loader::ComponentsLoader;
use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::honkai::config::Config;
use iced::widget::{button, column, container, progress_bar, row, stack, text, vertical_space};
use iced::{Alignment, Command, Element, Length};

use super::finish::FinishView;
use super::tos_warning::TosWarningView;
use super::welcome::WelcomeView;
use crate::{CONFIG, FIRST_RUN_FILE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome = 0,
    TosWarning = 1,
    Dependencies = 2,
    DefaultPaths = 3,
    DownloadComponents = 4,
    Finish = 5,
}

#[derive(Debug, Clone)]
pub struct ToastData {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FirstRunMessage {
    NextStep,
    PreviousStep,
    GoTo(Step),
    AgreeTos,
    SetLoading(Option<String>),
    SyncComponentsFinished(Result<(), String>),
    ShowToast(ToastData),
    DismissToast,
    OpenDebugFile,
    FinishAndLaunch,
    ExitApp,
}

pub struct FirstRunState {
    pub current_step: Step,
    pub loading: Option<String>,
    pub toast: Option<ToastData>,
}

impl FirstRunState {
    pub fn new() -> (Self, Command<FirstRunMessage>) {
        (
            Self {
                current_step: Step::Welcome,
                loading: None,
                toast: None,
            },
            Command::none(),
        )
    }

    pub fn update(&mut self, message: FirstRunMessage) -> Command<FirstRunMessage> {
        match message {
            FirstRunMessage::NextStep => {
                match self.current_step {
                    Step::Welcome => self.current_step = Step::TosWarning,
                    Step::TosWarning => {
                        if TosWarningView::check_dependencies() {
                            self.current_step = Step::DefaultPaths;
                        } else {
                            self.current_step = Step::Dependencies;
                        }
                    }
                    Step::Dependencies => self.current_step = Step::DefaultPaths,
                    Step::DefaultPaths => {
                        self.loading = Some("Updating components index...".to_string());
                        return Command::perform(Self::sync_components_index(), FirstRunMessage::SyncComponentsFinished);
                    }
                    Step::DownloadComponents => self.current_step = Step::Finish,
                    Step::Finish => {}
                }
                Command::none()
            }

            FirstRunMessage::PreviousStep => {
                match self.current_step {
                    Step::TosWarning => self.current_step = Step::Welcome,
                    Step::Dependencies => self.current_step = Step::TosWarning,
                    Step::DefaultPaths => self.current_step = Step::TosWarning,
                    Step::DownloadComponents => self.current_step = Step::DefaultPaths,
                    Step::Finish => self.current_step = Step::DownloadComponents,
                    _ => {}
                }
                Command::none()
            }

            FirstRunMessage::GoTo(step) => {
                self.current_step = step;
                Command::none()
            }

            FirstRunMessage::AgreeTos => {
                self.update(FirstRunMessage::NextStep)
            }

            FirstRunMessage::SetLoading(status) => {
                self.loading = status;
                Command::none()
            }

            FirstRunMessage::SyncComponentsFinished(res) => {
                self.loading = None;
                self.current_step = Step::DownloadComponents;

                if let Err(err) = res {
                    return Command::perform(
                        async move {
                            ToastData {
                                title: "Failed to sync components index".into(),
                                description: Some(err),
                            }
                        },
                        FirstRunMessage::ShowToast,
                    );
                }
                Command::none()
            }

            FirstRunMessage::ShowToast(toast) => {
                self.toast = Some(toast);
                Command::none()
            }

            FirstRunMessage::DismissToast => {
                self.toast = None;
                Command::none()
            }

            FirstRunMessage::OpenDebugFile => {
                let _ = open::that(crate::DEBUG_FILE.as_os_str());
                Command::none()
            }

            FirstRunMessage::FinishAndLaunch => {
                if FIRST_RUN_FILE.exists() {
                    let _ = std::fs::remove_file(FIRST_RUN_FILE.as_path());
                }

                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                }
                std::process::exit(0);
            }

            FirstRunMessage::ExitApp => {
                std::process::exit(0);
            }
        }
    }

    async fn sync_components_index() -> Result<(), String> {
        tokio::task::spawn_blocking(|| {
            let config = Config::get().unwrap_or_else(|_| CONFIG.clone());
            let components = ComponentsLoader::new(config.components.path);

            match components.is_sync(&config.components.servers) {
                Ok(Some(_)) => Ok(()),
                Ok(None) => {
                    for host in &CONFIG.components.servers {
                        if components.sync(host).is_ok() {
                            return Ok(());
                        }
                    }
                    Err("All component mirror servers failed to respond.".into())
                }
                Err(e) => Err(e.to_string()),
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub fn view(&self) -> Element<'_, FirstRunMessage> {
        if let Some(loading_msg) = &self.loading {
            return container(
                column![
                    text("Please wait...").size(24),
                    vertical_space().height(12),
                    text(loading_msg).size(14),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }

        let step_content: Element<'_, FirstRunMessage> = match self.current_step {
            Step::Welcome => WelcomeView::view(),
            Step::TosWarning => TosWarningView::view(),
            Step::Dependencies => container(
                column![
                    text("Missing Dependencies").size(22),
                    text("Please install: git, dwebp, 7z (or 7za) using your package manager.").size(14),
                    vertical_space().height(20),
                    button(text("Re-check").size(14)).on_press(FirstRunMessage::NextStep)
                ]
                .align_items(Alignment::Center),
            )
            .center_x()
            .center_y()
            .into(),
            Step::DefaultPaths => container(
                column![
                    text("Default Paths Configuration").size(22),
                    text("Paths are verified and configured.").size(14),
                    vertical_space().height(20),
                    button(text("Next").size(14)).on_press(FirstRunMessage::NextStep)
                ]
                .align_items(Alignment::Center),
            )
            .center_x()
            .center_y()
            .into(),
            Step::DownloadComponents => container(
                column![
                    text("Download Game Components").size(22),
                    text("You can download Wine / DXVK versions now or later in Settings.").size(14),
                    vertical_space().height(20),
                    button(text("Finish Setup").size(14)).on_press(FirstRunMessage::NextStep)
                ]
                .align_items(Alignment::Center),
            )
            .center_x()
            .center_y()
            .into(),
            Step::Finish => FinishView::view(),
        };

        let indicator = row(
            (0..=5)
                .map(|i| {
                    let is_active = self.current_step as usize == i;
                    let dot_size = if is_active { 10.0 } else { 6.0 };
                    let label = if is_active { "●" } else { "○" };
                    text(label).size(dot_size).into()
                })
                .collect::<Vec<Element<'_, FirstRunMessage>>>(),
        )
        .spacing(8)
        .align_items(Alignment::Center);

        let main_layout = column![
            container(step_content).width(Length::Fill).height(Length::Fill),
            container(indicator).padding(16).center_x()
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(toast) = &self.toast {
            let toast_box = container(
                row![
                    column![
                        text(&toast.title).size(14),
                        if let Some(desc) = &toast.description {
                            text(desc).size(11)
                        } else {
                            text("").size(0)
                        }
                    ],
                    button(text("Log").size(11)).on_press(FirstRunMessage::OpenDebugFile),
                    button(text("✕").size(11)).on_press(FirstRunMessage::DismissToast)
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            )
            .padding(12)
            .style(iced::theme::Container::Box);

            stack![
                main_layout,
                container(toast_box)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .align_y(iced::alignment::Vertical::Bottom)
                    .padding(20)
            ]
            .into()
        } else {
            main_layout.into()
        }
    }
}
