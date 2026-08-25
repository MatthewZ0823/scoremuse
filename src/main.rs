#[macro_use]
extern crate num_derive;

use crate::bar::Bar;
use crate::font::{FontMeta, load_font};
use crate::note_or_rest::{BaseDuration, NoteOrRest};
use crate::pages::{loading_page, score_editing_page};
use crate::pitch::{Accidental, Pitch, PitchClass};
use crate::staff::{Staff, StaffEl};
use iced::Element;
use iced::Task;

mod bar;
mod colors;
mod constants;
mod font;
mod midi;
mod note_or_rest;
mod pages;
mod pitch;
mod playback;
mod staff;
mod utils;

enum App {
    Loading(loading_page::LoadingPage),
    ScoreEditing(score_editing_page::ScoreEditingPage),
}

impl Default for App {
    fn default() -> Self {
        Self::Loading(Default::default())
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            App::Loading(loading_page) => match message {
                Message::LoadingMessage(loading_msg) => {
                    let font_meta = loading_page.update(loading_msg);

                    let staff = Staff {
                        bars: vec![
                            Bar::new(vec![
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::E, 5, Some(Accidental::Sharp))),
                                    BaseDuration(1),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::G, 4, Some(Accidental::Flat))),
                                    BaseDuration(1),
                                ),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4, Some(Accidental::Natural))),
                                    BaseDuration(2),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::G, 4, None)),
                                    BaseDuration(2),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::A, 4, None)),
                                    BaseDuration(1),
                                ),
                            ]),
                            Bar::new(vec![
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4, None)),
                                    BaseDuration(3),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::E, 5, None)),
                                    BaseDuration(4),
                                ),
                                NoteOrRest::new(
                                    Some(Pitch::new(PitchClass::F, 4, None)),
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

                    let score_editing_state =
                        score_editing_page::ScoreEditingPage::new(StaffEl::new(staff, font_meta));
                    *self = App::ScoreEditing(score_editing_state);

                    Task::none()
                }
                Message::ScoreEditingMessage(..) => Task::none(),
            },
            App::ScoreEditing(score_editing_page) => match message {
                Message::ScoreEditingMessage(score_editing_message) => score_editing_page
                    .update(score_editing_message)
                    .map(Message::ScoreEditingMessage),
                Message::LoadingMessage(..) => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            App::ScoreEditing(score_editing_page) => score_editing_page
                .view()
                .map(|msg| Message::ScoreEditingMessage(msg)),
            App::Loading(loading_page) => {
                loading_page.view().map(|msg| Message::LoadingMessage(msg))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    LoadingMessage(loading_page::LoadingMessage),
    ScoreEditingMessage(score_editing_page::ScoreEditingMessage),
}

fn boot() -> (App, iced::Task<Message>) {
    (
        App::default(),
        load_font("Bravura".to_string())
            .map(|m| Message::LoadingMessage(loading_page::LoadingMessage::FontLoaded(m))),
    )
}

fn main() -> iced::Result {
    // synthesize_staff();
    iced::application(boot, App::update, App::view).run()
}
