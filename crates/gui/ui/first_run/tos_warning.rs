use anime_launcher_sdk::is_available;
use iced::widget::{button, column, container, row, text, vertical_space};
use iced::{Alignment, Element, Length};

use super::main::FirstRunMessage;

pub struct TosWarningView;

impl TosWarningView {
    pub fn check_dependencies() -> bool {
        is_available("git") && is_available("dwebp") && (is_available("7z") || is_available("7za"))
    }

    pub fn view<'a>() -> Element<'a, FirstRunMessage> {
        let title = text("Terms of Service Notice").size(26);
        let warning_text = text(
            "Using custom launchers or running the game through Wine/Proton on Linux \
            may technically violate the game's Terms of Service.\n\n\
            While the community has not observed bans solely for using this launcher, \
            you proceed at your own risk.",
        )
        .size(14)
        .horizontal_alignment(iced::alignment::Horizontal::Center);

        let buttons = row![
            button(text("Exit").size(14))
                .padding([8, 24])
                .on_press(FirstRunMessage::ExitApp),
            button(text("I Agree").size(14))
                .padding([8, 24])
                .on_press(FirstRunMessage::AgreeTos),
        ]
        .spacing(16);

        let content = column![
            vertical_space().height(Length::FillPortion(1)),
            title,
            vertical_space().height(16),
            container(warning_text).max_width(550),
            vertical_space().height(Length::FillPortion(1)),
            buttons,
            vertical_space().height(20)
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
