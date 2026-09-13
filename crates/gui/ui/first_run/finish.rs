use iced::widget::{button, column, container, row, text, vertical_space};
use iced::{Alignment, Element, Length};

use super::main::FirstRunMessage;

pub struct FinishView;

impl FinishView {
    pub fn view<'a>() -> Element<'a, FirstRunMessage> {
        let title = text("Setup Completed!").size(28);
        let desc = text(
            "Configuration has been saved successfully.\n\
            Click Start to launch into the main application.",
        )
        .size(15)
        .horizontal_alignment(iced::alignment::Horizontal::Center);

        let actions = row![
            button(text("Exit").size(14))
                .padding([8, 24])
                .on_press(FirstRunMessage::ExitApp),
            button(text("Start Launcher").size(14))
                .padding([8, 28])
                .on_press(FirstRunMessage::FinishAndLaunch),
        ]
        .spacing(16);

        let content = column![
            vertical_space().height(Length::FillPortion(1)),
            title,
            vertical_space().height(16),
            desc,
            vertical_space().height(Length::FillPortion(1)),
            actions,
            vertical_space().height(24)
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
