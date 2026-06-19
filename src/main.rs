#[macro_use]
extern crate num_derive;

use crate::bar::Bar;
use crate::font::{FontMeta, load_font};
use crate::note_or_rest::NoteOrRest;
use crate::pitch::{Pitch, PitchClass};
use crate::staff::{Staff, StaffEl, StaffIndex};
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::{canvas, column, text};

mod bar;
mod canvas_svg;
mod colors;
mod constants;
mod font;
mod note_or_rest;
mod pitch;
mod staff;
mod utils;

const DEBUG: bool = false;

#[derive(Default)]
struct App {
    staff: Option<StaffEl>, // None when loading font
}

impl App {
    fn update(&mut self, message: Message) {
        match &mut self.staff {
            None => match message {
                Message::FontLoaded(res) => {
                    let font_meta = res.unwrap(); // TODO: Better error handling
                    let staff = Staff {
                        bars: vec![
                            Bar::new(vec![
                                NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), 1),
                                NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), 1),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 2),
                                NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), 2),
                                NoteOrRest::new(Some(Pitch::new(PitchClass::A, 4)), 1),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 3),
                                NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), 4),
                                NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), 4),
                                NoteOrRest::new(None, 2),
                            ]),
                            Bar::new(vec![NoteOrRest::new(None, 0)]),
                            Bar::new(vec![
                                NoteOrRest::new(None, 1),
                                NoteOrRest::new(None, 2),
                                NoteOrRest::new(None, 3),
                                NoteOrRest::new(None, 4),
                                NoteOrRest::new(None, 5),
                                NoteOrRest::new(None, 5),
                            ]),
                        ],
                    };
                    self.staff = Some(StaffEl::new(staff, font_meta));
                }
                Message::SetNote(..) | Message::AddBar => (),
            },
            Some(staff) => match message {
                Message::AddBar => {
                    staff.add_bar();
                    staff.redraw();
                }
                Message::SetNote(staff_index, pitch) => {
                    staff.set_note(&staff_index, pitch);
                    staff.redraw();
                }
                Message::FontLoaded(..) => (),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.staff {
            Some(staff) => {
                let staff_canvas = canvas(staff).width(Fill).height(Fill);

                // The layout
                let interface: Element<_> = column![text("hello world").height(100), staff_canvas]
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
            None => text("loading font...").into(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<FontMeta, font::Error>),
    SetNote(StaffIndex, Pitch),
    AddBar,
}

fn boot() -> (App, iced::Task<Message>) {
    (
        App::default(),
        load_font("Bravura".to_string()).map(|m| Message::FontLoaded(m)),
    )
}

fn main() -> iced::Result {
    iced::application(boot, App::update, App::view).run()
}
