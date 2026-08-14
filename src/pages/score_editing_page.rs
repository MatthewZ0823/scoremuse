use iced::{
    Element, Fill, Padding, Task, alignment,
    futures::{SinkExt, channel::mpsc},
    widget::{Button, button, canvas, column, row, text},
};
use rodio::MixerDeviceSink;

use crate::{
    note_or_rest::BaseDuration,
    playback::{AudioCommand, AudioEvent, play_midi},
    staff::{StaffEl, StaffInteractionMsg, StaffInteractionState, handle_staff_interaction_msg},
};
use std::sync::Arc;

pub struct ScoreEditingPage {
    staff: StaffEl,
    playback: PlaybackState,
}

impl ScoreEditingPage {
    pub fn view(&self) -> Element<'_, ScoreEditingMessage> {
        let staff = &self.staff;
        let staff_canvas = canvas(staff).width(Fill).height(Fill);

        let edit_note_buttons_active = staff.staff_interaction.get_selected().is_some()
            || matches!(staff.staff_interaction, StaffInteractionState::Dragging(..));

        // The layout
        let change_duration_button =
            |base_duration: BaseDuration| -> Button<'_, ScoreEditingMessage> {
                let glyph = match base_duration.0 {
                    0 => "\u{E1D2}",
                    1 => "\u{E1D3}",
                    2 => "\u{E1D5}",
                    3 => "\u{E1D7}",
                    4 => "\u{E1D9}",
                    5 => "\u{E1DB}",
                    _ => panic!(),
                };
                let on_press = if edit_note_buttons_active {
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

        let toggle_rest_button_msg = if edit_note_buttons_active {
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
        .spacing(4.);

        let playback_controls = row![{
            let (glyph, message) = match self.playback.playback_status {
                PlayingStatus::Paused(..) | PlayingStatus::PlaybackNotStarted => {
                    ("\u{23F5}", ScoreEditingMessage::PlayButtonClick)
                }
                PlayingStatus::Playing(..) => ("\u{23F8}", ScoreEditingMessage::PauseButtonClick),
            };

            button(text(glyph)).on_press(message)
        }];

        let add_bar_button = button("Add Bar").on_press(ScoreEditingMessage::AddBarButtonClick);

        let controls = row![duration_controls, playback_controls, add_bar_button]
            .spacing(4.)
            .padding([10., 0.]);

        column![controls, staff_canvas]
            .height(Fill)
            .width(Fill)
            .into()
    }

    pub fn update(
        &mut self,
        score_editing_message: ScoreEditingMessage,
    ) -> Task<ScoreEditingMessage> {
        let staff = &mut self.staff;
        let playback = &mut self.playback;

        match score_editing_message {
            ScoreEditingMessage::AddBarButtonClick => {
                staff.add_bar();
                staff.redraw();
                Task::none()
            }
            ScoreEditingMessage::StaffInteractionMsg(staff_interaction_msg) => {
                handle_staff_interaction_msg(staff_interaction_msg, staff);
                Task::none()
            }
            ScoreEditingMessage::BaseDurationButtonClick(base_duration) => {
                if let Some(staff_idx) = staff.staff_interaction.get_selected().cloned() {
                    staff.set_note_base_duration(&staff_idx, base_duration);
                    staff.staff_interaction = StaffInteractionState::NONE;
                    staff.redraw();
                }
                Task::none()
            }
            ScoreEditingMessage::ToggleRestButtonClick => {
                if let Some(staff_idx) = staff.staff_interaction.get_selected().cloned() {
                    staff.set_note_pitch(&staff_idx, None);
                    staff.staff_interaction = StaffInteractionState::NONE;
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

    pub fn new(staff: StaffEl) -> Self {
        Self {
            staff,
            playback: Default::default(),
        }
    }
}

/// Messages when the app is in the score editing state
#[derive(Debug, Clone)]
pub enum ScoreEditingMessage {
    StaffInteractionMsg(StaffInteractionMsg),
    BaseDurationButtonClick(BaseDuration),
    ToggleRestButtonClick,
    PlayButtonClick,
    PauseButtonClick,
    PausedPlayback,
    PlayingPlayback,
    PlaybackDone,
    AddBarButtonClick,
}

struct PlaybackState {
    playback_status: PlayingStatus,
    audio_handle: Arc<MixerDeviceSink>,
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
