use crate::staff::{Pitch, Staff};
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::canvas;
use iced::widget::{button, column, text};

mod canvas_svg;
mod staff;

const DEBUG: bool = true;

#[derive(Default)]
struct App {
    value: i64,
    staff: Staff,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::AddNote(note) => {
                self.staff.notes.push(note);
                self.staff.redraw();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The buttons
        let increment = button("+").on_press(Message::Increment);
        let decrement = button("-").on_press(Message::Decrement);

        // The number
        let counter = text(self.value).size(100);

        let staff = canvas(&self.staff).width(Fill).height(Fill);

        // The layout
        let interface: Element<_> = column![increment, counter, decrement, staff]
            .height(Fill)
            .width(Fill)
            .into();

        let explained = if DEBUG {
            interface.explain(Color::BLACK)
        } else {
            interface
        };

        explained
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
    AddNote(Pitch),
}

fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

#[test]
fn it_counts_properly() {
    let mut counter = App::default();
    counter.value = 0;

    counter.update(Message::Increment);
    counter.update(Message::Increment);
    counter.update(Message::Decrement);

    assert_eq!(counter.value, 1);
}
