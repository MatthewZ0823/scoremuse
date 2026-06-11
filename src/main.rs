use crate::bar::Bar;
use crate::constants::MUSIC_FONT;
use crate::note_or_rest::NoteOrRest;
use crate::pitch::{Pitch, PitchClass};
use crate::staff::{Staff, StaffEl, StaffIndex};
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::{button, column, text};
use iced::widget::{canvas, svg};

mod bar;
mod canvas_svg;
mod colors;
mod constants;
mod note_or_rest;
mod pitch;
mod staff;
mod utils;

const DEBUG: bool = false;

struct App {
    value: i64,
    staff: StaffEl,
}

impl Default for App {
    fn default() -> Self {
        let staff = Staff {
            bars: vec![
                Bar::new(vec![
                    NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), 2),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), 2),
                ]),
                Bar::new(vec![
                    NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 3),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), 3),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::A, 4)), 2),
                ]),
                Bar::new(vec![
                    NoteOrRest::new(None, 2),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 4),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), 5),
                    NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 5),
                    NoteOrRest::new(None, 3),
                ]),
                Bar::new(vec![NoteOrRest::new(None, 1)]),
            ],
        };
        let staff_el = StaffEl::new(staff);

        App {
            value: 0,
            staff: staff_el,
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
            Message::AddBar => {
                self.staff.add_bar();
                self.staff.redraw();
            }
            Message::SetNote(staff_index, pitch) => {
                self.staff.set_note(&staff_index, pitch);
                self.staff.redraw();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The buttons
        let increment = button("+").on_press(Message::Increment);
        let decrement = button("-").on_press(Message::Decrement);

        let half_note = svg("src/assets/half_note.svg").width(20.);

        // The number
        let counter = text(self.value).size(100);

        let staff = canvas(&self.staff).width(Fill).height(Fill);

        let test = text("\u{E050}").font(MUSIC_FONT);

        // The layout
        let interface: Element<_> = column![increment, counter, decrement, test, half_note, staff]
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
    SetNote(StaffIndex, Pitch),
    AddBar,
}

fn main() -> iced::Result {
    // iced::run(App::update, App::view)

    iced::application(App::default, App::update, App::view)
        .font(include_bytes!("../fonts/Bravura.otf").as_slice())
        .run()
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
