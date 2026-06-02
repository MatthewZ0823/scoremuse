use crate::staff::{Bar, NoteOrRest, PitchClass};
use crate::staff::{Pitch, Staff};
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::canvas;
use iced::widget::{button, column, text};

mod canvas_svg;
mod note;
mod staff;

const DEBUG: bool = true;

struct App {
    value: i64,
    staff: Staff,
}

impl Default for App {
    fn default() -> Self {
        let mut staff = Staff::default();
        staff.bars = vec![
            Bar::new(vec![
                NoteOrRest::new(Some((PitchClass::E, 5)), 2),
                NoteOrRest::new(Some((PitchClass::G, 4)), 2),
            ]),
            Bar::new(vec![
                NoteOrRest::new(Some((PitchClass::F, 4)), 3),
                NoteOrRest::new(Some((PitchClass::G, 4)), 3),
                NoteOrRest::new(Some((PitchClass::A, 4)), 2),
            ]),
            Bar::new(vec![
                NoteOrRest::new(None, 2),
                NoteOrRest::new(Some((PitchClass::F, 4)), 4),
                NoteOrRest::new(Some((PitchClass::E, 5)), 5),
                NoteOrRest::new(Some((PitchClass::F, 4)), 5),
                NoteOrRest::new(None, 3),
            ]),
            Bar::new(vec![NoteOrRest::new(None, 1)]),
        ];

        App {
            value: 0,
            staff: staff,
        }
    }
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
            Message::AddNote(_note) => {
                // self.staff.notes.push(note);
                // self.staff.redraw();
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
