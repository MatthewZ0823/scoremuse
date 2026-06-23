#[macro_use]
extern crate num_derive;

use crate::bar::Bar;
use crate::font::{FontMeta, load_font};
use crate::note_or_rest::{BaseDuration, NoteOrRest};
use crate::pitch::{Pitch, PitchClass};
use crate::staff::{
    Staff, StaffEl, StaffInteractionMsg, StaffInteractionState, handle_staff_interaction_msg,
};
use iced::Fill;
use iced::widget::{Button, button, canvas, column, float, row, text};
use iced::{Color, alignment};
use iced::{Element, Vector};

mod bar;
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
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::E, 5)),
                                    BaseDuration(1),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::G, 4)),
                                    BaseDuration(1),
                                ),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4)),
                                    BaseDuration(2),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::G, 4)),
                                    BaseDuration(2),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::A, 4)),
                                    BaseDuration(1),
                                ),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4)),
                                    BaseDuration(3),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::E, 5)),
                                    BaseDuration(4),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4)),
                                    BaseDuration(4),
                                ),
                                NoteOrRest::new(None, BaseDuration(2)),
                            ]),
                            Bar::new(vec![NoteOrRest::new(None, BaseDuration(0))]),
                            Bar::new(vec![
                                NoteOrRest::new(None, BaseDuration(1)),
                                NoteOrRest::new(None, BaseDuration(2)),
                                NoteOrRest::new(None, BaseDuration(3)),
                                NoteOrRest::new(None, BaseDuration(4)),
                                NoteOrRest::new(None, BaseDuration(5)),
                                NoteOrRest::new(None, BaseDuration(5)),
                            ]),
                        ],
                    };
                    self.staff = Some(StaffEl::new(staff, font_meta));
                }
                Message::StaffInteractionMsg(..)
                | Message::BaseDurationButtonClick(..)
                | Message::AddBar => (),
                Message::NoOp => (),
            },
            Some(staff) => match message {
                Message::AddBar => {
                    staff.add_bar();
                    staff.redraw();
                }
                Message::StaffInteractionMsg(staff_interaction_msg) => {
                    handle_staff_interaction_msg(staff_interaction_msg, staff);
                }
                Message::BaseDurationButtonClick(base_duration) => {
                    if let StaffInteractionState::Selected(staff_idx, _) = staff.staff_interaction {
                        staff.set_note_base_duration(&staff_idx, base_duration);
                        staff.staff_interaction = StaffInteractionState::None;
                        staff.redraw();
                    }
                }
                Message::FontLoaded(..) => (),
                Message::NoOp => (),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.staff {
            Some(staff) => {
                let staff_canvas = canvas(staff).width(Fill).height(Fill);

                // The layout
                let glyph_button = |base_duration: BaseDuration| -> Button<'_, Message> {
                    let glyph = match base_duration.0 {
                        0 => "\u{E1D2}",
                        1 => "\u{E1D3}",
                        2 => "\u{E1D5}",
                        3 => "\u{E1D7}",
                        4 => "\u{E1D9}",
                        5 => "\u{E1DB}",
                        _ => panic!(),
                    };

                    button(
                        float(
                            text(glyph)
                                .align_y(alignment::Vertical::Bottom)
                                .font(staff.get_font())
                                .size(30.)
                                .color(Color::from_rgb(1., 1., 1.)),
                        )
                        .translate(|_, _| Vector::new(0., 10.)),
                    )
                    .on_press(Message::BaseDurationButtonClick(base_duration))
                };

                let duration_controls = row![
                    glyph_button(BaseDuration(0)),
                    glyph_button(BaseDuration(1)),
                    glyph_button(BaseDuration(2)),
                    glyph_button(BaseDuration(3)),
                    glyph_button(BaseDuration(4)),
                    glyph_button(BaseDuration(5)),
                ]
                .spacing(4.)
                .padding([0., 4.])
                .height(100.);

                let interface: Element<_> = column![
                    text("hello world").height(100),
                    duration_controls,
                    staff_canvas
                ]
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
    StaffInteractionMsg(StaffInteractionMsg),
    BaseDurationButtonClick(BaseDuration),
    AddBar,
    NoOp,
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
