#[macro_use]
extern crate num_derive;

use crate::bar::Bar;
use crate::font::{FontMeta, load_font};
use crate::note_or_rest::{BaseDuration, NoteOrRest};
use crate::pitch::{Pitch, PitchClass};
use crate::playback::synthesize_staff;
use crate::staff::{
    Staff, StaffEl, StaffInteractionMsg, StaffInteractionState, handle_staff_interaction_msg,
};
use iced::Fill;
use iced::widget::{Button, button, canvas, column, container, float, row, text};
use iced::{Color, alignment};
use iced::{Element, Vector};

mod bar;
mod colors;
mod constants;
mod font;
mod note_or_rest;
mod pitch;
mod playback;
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
                Message::LoadingMessage(loading_msg) => handle_loading_message(self, loading_msg),
                Message::ScoreEditingMessage(..) => (),
            },
            Some(staff) => match message {
                Message::LoadingMessage(..) => (),
                Message::ScoreEditingMessage(score_editing_message) => {
                    handle_score_editing_message(staff, score_editing_message);
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.staff {
            Some(staff) => score_editing_view(staff).map(|msg| Message::ScoreEditingMessage(msg)),
            None => text("loading font...").into(),
        }
    }
}

fn score_editing_view(staff: &StaffEl) -> Element<'_, ScoreEditingMessage> {
    let staff_canvas = canvas(staff).width(Fill).height(Fill);

    // The layout
    let change_duration_button = |base_duration: BaseDuration| -> Button<'_, ScoreEditingMessage> {
        let glyph = match base_duration.0 {
            0 => "\u{E1D2}",
            1 => "\u{E1D3}",
            2 => "\u{E1D5}",
            3 => "\u{E1D7}",
            4 => "\u{E1D9}",
            5 => "\u{E1DB}",
            _ => panic!(),
        };
        let on_press = if let StaffInteractionState::Selected(..) = staff.staff_interaction {
            Some(ScoreEditingMessage::BaseDurationButtonClick(base_duration))
        } else {
            None
        };

        button(
            float(
                text(glyph)
                    .align_y(alignment::Vertical::Bottom)
                    .font(staff.get_font())
                    .size(30.), // .color(Color::from_rgb(1., 1., 1.)),
            )
            .translate(|_, _| Vector::new(0., 10.)),
        )
        .on_press_maybe(on_press)
    };

    let toggle_rest_button_msg =
        if let StaffInteractionState::Selected(..) = staff.staff_interaction {
            Some(ScoreEditingMessage::ToggleRestButtonClick)
        } else {
            None
        };
    let toggle_rest_button: Button<'_, ScoreEditingMessage> =
        button(text("\u{E4E5}").font(staff.get_font()).size(30.))
            .on_press_maybe(toggle_rest_button_msg);

    let duration_controls = row![
        change_duration_button(BaseDuration(0)),
        change_duration_button(BaseDuration(1)),
        change_duration_button(BaseDuration(2)),
        change_duration_button(BaseDuration(3)),
        change_duration_button(BaseDuration(4)),
        change_duration_button(BaseDuration(5)),
        toggle_rest_button,
    ]
    .spacing(4.)
    .padding([0., 4.]);

    let playback_controls =
        row![button(text("\u{25B6}")).on_press(ScoreEditingMessage::PlayButtonClick)];

    let controls = row![duration_controls, playback_controls].padding([10., 0.]);

    let interface: Element<_> = column![controls, staff_canvas]
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

fn handle_score_editing_message(staff: &mut StaffEl, score_editing_message: ScoreEditingMessage) {
    match score_editing_message {
        ScoreEditingMessage::AddBar => {
            staff.add_bar();
            staff.redraw();
        }
        ScoreEditingMessage::StaffInteractionMsg(staff_interaction_msg) => {
            handle_staff_interaction_msg(staff_interaction_msg, staff);
        }
        ScoreEditingMessage::BaseDurationButtonClick(base_duration) => {
            if let StaffInteractionState::Selected(staff_idx, _) = staff.staff_interaction {
                staff.set_note_base_duration(&staff_idx, base_duration);
                staff.staff_interaction = StaffInteractionState::None;
                staff.redraw();
            }
        }
        ScoreEditingMessage::ToggleRestButtonClick => {
            if let StaffInteractionState::Selected(staff_idx, _) = staff.staff_interaction {
                staff.set_note_pitch(&staff_idx, None);
                staff.staff_interaction = StaffInteractionState::None;
                staff.redraw();
            }
        }
        ScoreEditingMessage::PlayButtonClick => {
            synthesize_staff();
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    LoadingMessage(LoadingMessage),
    ScoreEditingMessage(ScoreEditingMessage),
}

/// Messages when the app is in the loading state
#[derive(Debug, Clone)]
enum LoadingMessage {
    FontLoaded(Result<FontMeta, font::Error>),
}

fn handle_loading_message(app: &mut App, loading_message: LoadingMessage) {
    match loading_message {
        LoadingMessage::FontLoaded(res) => {
            let font_meta = res.unwrap(); // TODO: Better error handling
            let staff = Staff {
                bars: vec![
                    Bar::new(vec![
                        NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), BaseDuration(1)),
                        NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), BaseDuration(1)),
                    ]),
                    Bar::new(vec![
                        NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), BaseDuration(2)),
                        NoteOrRest::new(Some(Pitch::new(PitchClass::G, 4)), BaseDuration(2)),
                        NoteOrRest::new(Some(Pitch::new(PitchClass::A, 4)), BaseDuration(1)),
                    ]),
                    Bar::new(vec![
                        NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), BaseDuration(3)),
                        NoteOrRest::new(Some(Pitch::new(PitchClass::E, 5)), BaseDuration(4)),
                        NoteOrRest::new(Some(Pitch::new(PitchClass::F, 4)), BaseDuration(4)),
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
            app.staff = Some(StaffEl::new(staff, font_meta));
        }
    }
}

/// Messages when the app is in the score editing state
#[derive(Debug, Clone)]
enum ScoreEditingMessage {
    StaffInteractionMsg(StaffInteractionMsg),
    BaseDurationButtonClick(BaseDuration),
    ToggleRestButtonClick,
    PlayButtonClick,
    AddBar,
}

fn boot() -> (App, iced::Task<Message>) {
    (
        App::default(),
        load_font("Bravura".to_string())
            .map(|m| Message::LoadingMessage(LoadingMessage::FontLoaded(m))),
    )
}

fn main() -> iced::Result {
    // synthesize_staff();
    iced::application(boot, App::update, App::view).run()
}
