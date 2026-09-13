use iced::widget::{button, column, container, image, row, text, vertical_space};
use iced::{Alignment, Element, Length};

use super::main::FirstRunMessage;

pub struct WelcomeView;

impl WelcomeView {
    pub fn view<'a>() -> Element<'a, FirstRunMessage> {
        let title = text("Unified Launcher").size(32);
        let desc = text(
            "Welcome to Unified Launcher.\nAn open-source, highly optimized launcher designed for Linux.",
        )
        .size(15)
        .horizontal_alignment(iced::alignment::Horizontal::Center);

        let content = column![
            vertical_space().height(Length::FillPortion(1)),
            title,
            vertical_space().height(16),
            desc,
            vertical_space().height(Length::FillPortion(1)),
            button(text("Continue").size(14))
                .padding([10, 32])
                .on_press(FirstRunMessage::NextStep)
        ]
        .spacing(12)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
