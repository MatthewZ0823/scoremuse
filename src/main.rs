#[macro_use]
extern crate num_derive;

use std::sync::Arc;

use crate::bar::Bar;
use crate::font::{FontMeta, load_font};
use crate::note_or_rest::{BaseDuration, NoteOrRest};
use crate::pitch::{Pitch, PitchClass};
use crate::playback::play_midi;
use crate::staff::{
    Staff, StaffEl, StaffInteractionMsg, StaffInteractionState, handle_staff_interaction_msg,
};
use iced::Element;
use iced::Fill;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::widget::{Button, button, canvas, column, row, text};
use iced::{Color, Padding, Task, alignment};
use rodio::MixerDeviceSink;

mod bar;
mod colors;
mod constants;
mod font;
mod midi;
mod note_or_rest;
mod pitch;
mod playback;
mod staff;
mod utils;

const DEBUG: bool = false;

#[derive(Default)]
enum App {
    #[default]
    Loading,
    ScoreEditing(ScoreEditingState),
}

struct ScoreEditingState {
    staff: StaffEl,
    playback: PlaybackState,
}

struct PlaybackState {
    playback_status: PlayingStatus,
    audio_handle: Arc<MixerDeviceSink>,
}

#[derive(Default)]
enum PlayingStatus {
    #[default]
    PlaybackNotStarted,
    Playing(mpsc::Sender<AudioCommand>),
    Paused(mpsc::Sender<AudioCommand>),
}

impl PlayingStatus {
    /// Toggle between `Playing` and `Paused`, does nothing if the state is not `Playing` or `Paused`
    fn toggle_playing(&mut self) {
        match self {
            PlayingStatus::Playing(..) => {
                // Need this `tmp` variable because of the borrow checker
                let mut tmp = PlayingStatus::default();
                core::mem::swap(self, &mut tmp);
                if let PlayingStatus::Playing(val) = tmp {
                    *self = PlayingStatus::Paused(val);
                }
            }
            PlayingStatus::Paused(..) => {
                // Need this `tmp` variable because of the borrow checker
                let mut tmp = PlayingStatus::default();
                core::mem::swap(self, &mut tmp);
                if let PlayingStatus::Paused(val) = tmp {
                    *self = PlayingStatus::Playing(val);
                }
            }
            _ => (),
        }
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            playback_status: Default::default(),
            audio_handle: Arc::new(
                rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream"),
            ),
        }
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            App::Loading => match message {
                Message::LoadingMessage(loading_msg) => {
                    handle_loading_message(self, loading_msg).map(Message::LoadingMessage)
                }
                Message::ScoreEditingMessage(..) => Task::none(),
            },
            App::ScoreEditing(score_editing_state) => match message {
                Message::LoadingMessage(..) => Task::none(),
                Message::ScoreEditingMessage(score_editing_message) => {
                    handle_score_editing_message(score_editing_state, score_editing_message)
                        .map(Message::ScoreEditingMessage)
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            App::ScoreEditing(score_editing_state) => {
                score_editing_view(score_editing_state).map(|msg| Message::ScoreEditingMessage(msg))
            }
            App::Loading => text("loading font...").into(),
        }
    }
}

fn score_editing_view(score_editing_state: &ScoreEditingState) -> Element<'_, ScoreEditingMessage> {
    let staff = &score_editing_state.staff;
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
        let on_press = if let StaffInteractionState::Selected(..) = &staff.staff_interaction {
            Some(ScoreEditingMessage::BaseDurationButtonClick(base_duration))
        } else {
            None
        };

        button(
            text(glyph)
                .align_y(alignment::Vertical::Bottom)
                .font(staff.get_font())
                .size(30.),
        )
        .padding(Padding {
            top: 20.,
            left: 5.,
            right: 5.,
            bottom: -5.,
        })
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

    let playback_controls = row![{
        let (glyph, message) = match score_editing_state.playback.playback_status {
            PlayingStatus::Paused(..) | PlayingStatus::PlaybackNotStarted => {
                ("\u{23F5}", ScoreEditingMessage::PlayButtonClick)
            }
            PlayingStatus::Playing(..) => ("\u{23F8}", ScoreEditingMessage::PauseButtonClick),
        };

        button(text(glyph)).on_press(message)
    }];

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

fn handle_score_editing_message(
    score_editing_state: &mut ScoreEditingState,
    score_editing_message: ScoreEditingMessage,
) -> Task<ScoreEditingMessage> {
    let staff = &mut score_editing_state.staff;
    let playback = &mut score_editing_state.playback;

    match score_editing_message {
        ScoreEditingMessage::AddBar => {
            staff.add_bar();
            staff.redraw();
            Task::none()
        }
        ScoreEditingMessage::StaffInteractionMsg(staff_interaction_msg) => {
            handle_staff_interaction_msg(staff_interaction_msg, staff);
            Task::none()
        }
        ScoreEditingMessage::BaseDurationButtonClick(base_duration) => {
            if let StaffInteractionState::Selected(staff_idx, _) = staff.staff_interaction {
                staff.set_note_base_duration(&staff_idx, base_duration);
                staff.staff_interaction = StaffInteractionState::None;
                staff.redraw();
            }
            Task::none()
        }
        ScoreEditingMessage::ToggleRestButtonClick => {
            if let StaffInteractionState::Selected(staff_idx, _) = staff.staff_interaction {
                staff.set_note_pitch(&staff_idx, None);
                staff.staff_interaction = StaffInteractionState::None;
                staff.redraw();
            }
            Task::none()
        }
        ScoreEditingMessage::PlaybackDone => {
            playback.playback_status = PlayingStatus::PlaybackNotStarted;
            Task::none()
        }
        ScoreEditingMessage::PlayButtonClick => match &playback.playback_status {
            PlayingStatus::PlaybackNotStarted => {
                let (sender, receiver) = mpsc::channel(32);
                playback.playback_status = PlayingStatus::Playing(sender);

                Task::run(
                    play_midi(
                        playback.audio_handle.clone(),
                        staff.get_midi_messages(),
                        receiver,
                    ),
                    |audio_event| match audio_event {
                        AudioEvent::PlaybackDone => ScoreEditingMessage::PlaybackDone,
                    },
                )
            }
            PlayingStatus::Paused(sender) => {
                let mut s = sender.clone();
                Task::future(async move { s.send(AudioCommand::Resume).await }).then({
                    |res| match res {
                        Ok(_) => Task::done(ScoreEditingMessage::PlayingPlayback),
                        Err(_) => Task::none(),
                    }
                })
            }
            PlayingStatus::Playing(..) => Task::none(),
        },
        ScoreEditingMessage::PauseButtonClick => match &mut playback.playback_status {
            PlayingStatus::Playing(sender) => {
                let mut s = sender.clone();
                Task::future(async move { s.send(AudioCommand::Pause).await }).then({
                    |res| match res {
                        Ok(_) => Task::done(ScoreEditingMessage::PausedPlayback),
                        Err(_) => Task::none(),
                    }
                })
            }
            PlayingStatus::PlaybackNotStarted | PlayingStatus::Paused(..) => Task::none(),
        },
        ScoreEditingMessage::PausedPlayback => {
            match &playback.playback_status {
                PlayingStatus::Playing(_sender) => {
                    playback.playback_status.toggle_playing();
                }
                PlayingStatus::PlaybackNotStarted | PlayingStatus::Paused(..) => (),
            }
            Task::none()
        }
        ScoreEditingMessage::PlayingPlayback => {
            match &playback.playback_status {
                PlayingStatus::Paused(_sender) => {
                    playback.playback_status.toggle_playing();
                }
                PlayingStatus::PlaybackNotStarted | PlayingStatus::Playing(..) => (),
            }
            Task::none()
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

#[derive(Debug)]
pub enum AudioCommand {
    Resume,
    Pause,
}

pub enum AudioEvent {
    /// Reached the end of playback
    PlaybackDone,
}

fn handle_loading_message(app: &mut App, loading_message: LoadingMessage) -> Task<LoadingMessage> {
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
            let score_editing_state = ScoreEditingState {
                staff: StaffEl::new(staff, font_meta),
                playback: PlaybackState::default(),
            };
            *app = App::ScoreEditing(score_editing_state);

            Task::none()
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
    PauseButtonClick,
    PausedPlayback,
    PlayingPlayback,
    PlaybackDone,
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
