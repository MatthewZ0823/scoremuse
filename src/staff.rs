use core::f32;
use std::array::from_fn;

use crate::canvas_svg::{CanvasSVG, Positioning::*, SizingMode::*};
use iced::widget::Action;
use iced::widget::canvas::{self};
use iced::{Point, Rectangle, Renderer, Theme, mouse};

use crate::Message;

const BARLINE_Y_SPACING: f32 = 20.;
const NOTE_Y_SPACING: f32 = BARLINE_Y_SPACING / 2.;

const TREBLE_CLEF_ASPECT_RATIO: f32 = 95.116 / 153.12;
const TREBLE_CLEF_PATH: &str = "src/assets/treble_clef.svg";

const FILLED_NOTE_HEAD_ASPECT_RATIO: f32 = 260. / 200.;
const FILLED_NOTE_HEAD_PATH: &str = "src/assets/filled_note_head.svg";

#[derive(Debug, Default)]
pub struct Staff {
    cache: canvas::Cache,
    pub notes: Vec<Pitch>,
}

impl Staff {
    pub fn redraw(&self) {
        self.cache.clear();
    }
}

#[derive(Default)]
pub struct StaffInternal {
    bar_lines: Option<[canvas::Path; 5]>,
    hovering: Option<Pitch>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum PitchClass {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
pub type Pitch = (PitchClass, u8);

fn y_offset_to_pitch(y: f32) -> Pitch {
    let note_space: f32 = 3. - y / NOTE_Y_SPACING;
    let mut pitch_class_num = (note_space % 7.).round() as i32;
    if pitch_class_num < 0 {
        pitch_class_num += 7;
    }
    let pitch_class = match pitch_class_num {
        0 => PitchClass::C,
        1 => PitchClass::D,
        2 => PitchClass::E,
        3 => PitchClass::F,
        4 => PitchClass::G,
        5 => PitchClass::A,
        6 => PitchClass::B,
        _ => panic!("Modular Arithmetic Error"),
    };
    let octave = ((note_space.round() / 7.).floor() + 5.) as u8;

    (pitch_class, octave)
}

fn pitch_to_y_offset(pitch: &Pitch) -> f32 {
    let (class, octave) = pitch;

    let class_offset = match class {
        PitchClass::C => 0.,
        PitchClass::D => 1.,
        PitchClass::E => 2.,
        PitchClass::F => 3.,
        PitchClass::G => 4.,
        PitchClass::A => 5.,
        PitchClass::B => 6.,
    };

    5. * BARLINE_Y_SPACING
        - class_offset * NOTE_Y_SPACING
        - ((*octave as f32) - 4.) * 7. * NOTE_Y_SPACING
}

fn create_bar_lines(bounds: &Rectangle) -> [canvas::Path; 5] {
    let width = bounds.size().width;

    from_fn(|i| {
        let y = i as f32 * BARLINE_Y_SPACING;
        let from = Point::new(0., y);
        let to = Point::new(width, y);

        canvas::Path::line(from, to)
    })
}

impl canvas::Program<Message> for Staff {
    type State = StaffInternal;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // TODO: Clean up update logic
        match event {
            iced::Event::Window(
                iced::window::Event::Opened { .. }
                | iced::window::Event::Resized(_)
                | iced::window::Event::Rescaled(_),
            ) => state.bar_lines = Some(create_bar_lines(&bounds)),
            _ => (),
        }

        if let iced::Event::Mouse(event) = event {
            match event {
                mouse::Event::CursorMoved { .. } => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        state.hovering = y_offset_to_pitch(cursor_position.y).into();
                    } else {
                        state.hovering = None;
                    }

                    self.cache.clear();
                    Some(Action::request_redraw())
                }
                mouse::Event::CursorLeft => {
                    state.hovering = None;
                    self.cache.clear();
                    Some(Action::request_redraw())
                }
                mouse::Event::ButtonPressed(button) => {
                    if *button == mouse::Button::Left {
                        match state.hovering {
                            Some(hovered) => Some(Action::publish(Message::AddNote(hovered))),
                            None => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            let bar_lines: &[canvas::Path; 5] = match &state.bar_lines {
                Some(bar_lines) => bar_lines,
                None => &create_bar_lines(&bounds),
            };

            bar_lines.iter().for_each(|path| {
                frame.stroke(&path, canvas::Stroke::default());
            });

            let treble_clef = CanvasSVG::new(
                TREBLE_CLEF_PATH,
                TREBLE_CLEF_ASPECT_RATIO,
                TopLeft(Point::new(0., -1.65 * BARLINE_Y_SPACING)),
                HeightOnly(7.5 * BARLINE_Y_SPACING),
            );
            treble_clef.draw_to_frame(frame);

            let note = |pitch| {
                CanvasSVG::new(
                    FILLED_NOTE_HEAD_PATH,
                    FILLED_NOTE_HEAD_ASPECT_RATIO,
                    Centered(Point::new(100., pitch_to_y_offset(pitch))),
                    HeightOnly(20.),
                )
            };
            self.notes.iter().for_each(|pitch| {
                note(pitch).draw_to_frame(frame);
            });

            match state.hovering {
                Some(hovered) => {
                    note(&hovered).draw_to_frame(frame);
                }
                None => (),
            }
        });

        vec![geom]
    }
}
