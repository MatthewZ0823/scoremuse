use core::f32;

use crate::canvas_svg::{CanvasSVG, Positioning::*, SizingMode::*};
use crate::note::{StemDirection, draw_note, draw_quarter_rest, draw_rect_rest};
use crate::pitch::{Pitch, PitchClass};
use iced::widget::Action;
use iced::widget::canvas::Style::Gradient;
use iced::widget::canvas::gradient::Linear;
use iced::widget::canvas::{self, Frame, LineCap, LineDash, LineJoin, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme, Vector, mouse};

use crate::Message;

const TOP_PADDIING: f32 = 30.;
const BARLINE_Y_SPACING: f32 = 15.;
// Padding in front of every bar before the first note
const BAR_PADDING: f32 = 2. * BARLINE_Y_SPACING;
const PREAMBLE_WIDTH: f32 = 6. * BARLINE_Y_SPACING;
const NOTE_Y_SPACING: f32 = BARLINE_Y_SPACING / 2.;

const TREBLE_CLEF_ASPECT_RATIO: f32 = 95.116 / 153.12;
const TREBLE_CLEF_PATH: &str = "src/assets/treble_clef.svg";

#[derive(Debug, Default)]
pub struct Staff {
    cache: canvas::Cache,
    pub bars: Vec<Bar>,
}

impl Staff {
    pub fn get_width(self: &Self) -> f32 {
        self.bars.iter().map(|b| b.get_width()).sum::<f32>() + PREAMBLE_WIDTH
    }
}

#[derive(Default)]
pub struct State {
    hovering: Option<Pitch>,
    hovering_new_bar: bool,
    new_bar_button_bounds: Rectangle,
}

#[derive(Debug)]
pub struct NoteOrRest {
    // When pitch is None its a rest
    pitch: Option<Pitch>,
    // 1 -> Whole Note, 2 -> Half Note, 3 - Quarter Note, ...
    duration: u8,
}

impl NoteOrRest {
    pub fn get_width(self: &Self) -> f32 {
        let width_factor = match self.duration {
            1 => 8.,
            2 => 4.,
            3 => 3.,
            _ => {
                if self.stem_down() {
                    2.
                } else {
                    3.
                }
            }
        };

        width_factor * BARLINE_Y_SPACING
    }

    fn stem_down(self: &Self) -> bool {
        match &self.pitch {
            Some(pitch) => *pitch > Pitch::new(PitchClass::B, 4),
            None => false,
        }
    }
}

impl NoteOrRest {
    pub fn new(pitch: Option<Pitch>, duration: u8) -> Self {
        NoteOrRest {
            pitch: pitch,
            duration: duration,
        }
    }
}

#[derive(Debug)]
pub struct Bar {
    // notes should be ordered by start
    notes: Vec<NoteOrRest>,
    width: f32,
}

impl Bar {
    pub fn new(notes: Vec<NoteOrRest>) -> Self {
        let width = notes.iter().map(|n| n.get_width()).sum::<f32>() + BAR_PADDING;
        Bar { notes, width }
    }

    pub fn get_width(self: &Self) -> f32 {
        self.width
    }
}

/// Returns the width of the rendered note
fn render_note(x: f32, note: &NoteOrRest, frame: &mut Frame) {
    match &note.pitch {
        None => match note.duration {
            1 => {
                draw_rect_rest(
                    frame,
                    BARLINE_Y_SPACING / 2.,
                    Point::new(x, BARLINE_Y_SPACING * 1.25),
                );
            }
            2 => {
                draw_rect_rest(
                    frame,
                    BARLINE_Y_SPACING / 2.,
                    Point::new(x, BARLINE_Y_SPACING * 1.75),
                );
            }
            3 => {
                draw_quarter_rest(
                    frame,
                    BARLINE_Y_SPACING * 2.5,
                    Point::new(x, BARLINE_Y_SPACING * 2.),
                );
            }
            _ => todo!(),
        },
        Some(pitch) => {
            let center = Point::new(x, pitch_to_y_offset(&pitch));
            let stem_direction = if center.y > 2. * BARLINE_Y_SPACING {
                StemDirection::UP
            } else {
                StemDirection::DOWN
            };
            draw_note(
                frame,
                note.duration,
                center,
                BARLINE_Y_SPACING,
                stem_direction,
            );
        }
    }
}

// Barlines not included
fn render_bar(x: f32, bar: &Bar, frame: &mut Frame) {
    let mut x_ = x + BAR_PADDING;
    for note in bar.notes.iter() {
        render_note(x_, note, frame);
        x_ += note.get_width();
    }

    let barline_path = Path::line(Point::new(x_, 0.), Point::new(x_, 4. * BARLINE_Y_SPACING));
    frame.stroke(&barline_path, canvas::Stroke::default());
}

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

    Pitch::new(pitch_class, octave)
}

fn pitch_to_y_offset(pitch: &Pitch) -> f32 {
    let Pitch {
        pitch_class,
        octave,
    } = pitch;

    let class_offset = match pitch_class {
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

fn draw_bar_lines(frame: &mut Frame, bounds: Rectangle, stroke: Stroke) {
    let spacing = bounds.height / 4.;
    for i in 0..5 {
        let y = i as f32 * spacing;
        let from = Point::new(bounds.x, y);
        let to = Point::new(bounds.x + bounds.width, y);
        let path = canvas::Path::line(from, to);
        frame.stroke(&path, stroke);
    }
}

impl canvas::Program<Message> for Staff {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // TODO: Clean up update logic
        let new_bar_width = 4. * BARLINE_Y_SPACING;
        let new_bar_button_bounds = Rectangle {
            x: self.get_width(),
            y: 0.,
            width: new_bar_width,
            height: 4. * BARLINE_Y_SPACING,
        };
        state.new_bar_button_bounds = new_bar_button_bounds;

        match event {
            iced::Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(button) => match button {
                    mouse::Button::Left => {
                        match cursor.position_in(bounds + Vector::new(0., TOP_PADDIING)) {
                            Some(position) => {
                                if state.new_bar_button_bounds.contains(position) {
                                    Some(canvas::Action::publish(Message::AddBar))
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                    _ => None,
                },
                mouse::Event::CursorMoved { .. } => {
                    match cursor.position_in(bounds + Vector::new(0., TOP_PADDIING)) {
                        Some(position) => {
                            let should_rerender;

                            if state.new_bar_button_bounds.contains(position) {
                                should_rerender = !state.hovering_new_bar;
                                state.hovering_new_bar = true;
                            } else {
                                should_rerender = state.hovering_new_bar;
                                state.hovering_new_bar = false;
                            }

                            if should_rerender {
                                self.cache.clear();
                                Some(Action::request_redraw())
                            } else {
                                None
                            }
                        }
                        None => {
                            if state.hovering_new_bar {
                                state.hovering_new_bar = false;
                                self.cache.clear();
                                Some(Action::request_redraw())
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
            iced::Event::Keyboard(_event) => None,
            iced::Event::Window(_event) => None,
            iced::Event::Touch(_event) => None,
            iced::Event::InputMethod(_event) => None,
        }

        // if let iced::Event::Mouse(event) = event {
        //     match event {
        //         mouse::Event::CursorMoved { .. } => {
        //             if let Some(cursor_position) = cursor.position_in(bounds) {
        //                 state.hovering = y_offset_to_pitch(cursor_position.y).into();
        //             } else {
        //                 state.hovering = None;
        //             }
        //
        //             self.cache.clear();
        //             Some(Action::request_redraw())
        //         }
        //         mouse::Event::CursorLeft => {
        //             state.hovering = None;
        //             self.cache.clear();
        //             Some(Action::request_redraw())
        //         }
        //         mouse::Event::ButtonPressed(button) => {
        //             if *button == mouse::Button::Left {
        //                 match state.hovering {
        //                     Some(hovered) => Some(Action::publish(Message::AddNote(hovered))),
        //                     None => None,
        //                 }
        //             } else {
        //                 None
        //             }
        //         }
        //         _ => None,
        //     }
        // } else {
        //     None
        // }
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
            frame.translate(Vector::new(0., TOP_PADDIING));
            let staff_bar_line_bounds = Rectangle {
                x: 0.,
                y: 0.,
                width: self.get_width(),
                height: 4. * BARLINE_Y_SPACING,
            };
            draw_bar_lines(frame, staff_bar_line_bounds, Stroke::default());

            // new bar icon
            let new_bar_stroke = Stroke {
                style: Gradient(
                    (Linear::new(
                        Point::new(state.new_bar_button_bounds.x, 0.),
                        Point::new(
                            state.new_bar_button_bounds.x + state.new_bar_button_bounds.width,
                            0.,
                        ),
                    )
                    .add_stop(0., Color::from_rgb(0.4, 0.4, 0.4))
                    .add_stop(1., Color::from_rgb(0.9, 0.9, 0.9)))
                    .into(),
                ),
                width: 1.,
                line_cap: LineCap::default(),
                line_join: LineJoin::default(),
                line_dash: LineDash::default(),
            };
            draw_bar_lines(frame, state.new_bar_button_bounds, new_bar_stroke);
            frame.fill(
                &Path::circle(
                    state.new_bar_button_bounds.center(),
                    BARLINE_Y_SPACING * 0.8,
                ),
                Color::from_rgb(1., 1., 1.),
            );
            let plus_path = Path::new(|b| {
                let r = BARLINE_Y_SPACING * 0.4;
                b.move_to(state.new_bar_button_bounds.center() + Vector::new(-r, 0.));
                b.line_to(state.new_bar_button_bounds.center() + Vector::new(r, 0.));

                b.move_to(state.new_bar_button_bounds.center() + Vector::new(0., -r));
                b.line_to(state.new_bar_button_bounds.center() + Vector::new(0., r));
            });
            frame.stroke(
                &plus_path,
                Stroke::default()
                    .with_width(3.)
                    .with_color(if state.hovering_new_bar {
                        Color::from_rgb(0.2, 0.4, 0.9)
                    } else {
                        Color::from_rgb(0.4, 0.4, 0.4)
                    })
                    .with_line_cap(LineCap::Round),
            );

            let treble_clef = CanvasSVG::new(
                TREBLE_CLEF_PATH,
                TREBLE_CLEF_ASPECT_RATIO,
                TopLeft(Point::new(0., -1.65 * BARLINE_Y_SPACING)),
                HeightOnly(7.5 * BARLINE_Y_SPACING),
            );
            treble_clef.draw(frame);

            {
                let mut x = PREAMBLE_WIDTH;
                for bar in self.bars.iter() {
                    render_bar(x, &bar, frame);
                    x += bar.get_width();
                }
            }

            // let note = |x, pitch| {
            //     CanvasSVG::new(
            //         FILLED_NOTE_HEAD_PATH,
            //         FILLED_NOTE_HEAD_ASPECT_RATIO,
            //         Centered(Point::new(x, pitch_to_y_offset(pitch))),
            //         HeightOnly(BARLINE_Y_SPACING * 1.1),
            //     )
            // };
            //
            // {
            //     let mut x = 100.;
            //     self.notes.iter().for_each(|pitch| {
            //         note(x, pitch).draw_to_frame(frame);
            //         x += 10.;
            //     });
            // }

            // match state.hovering {
            //     Some(hovered) => {
            //         note(100., &hovered).draw_to_frame(frame);
            //     }
            //     None => (),
            // }

            // let circle = canvas::Path::circle(Point { x: 0., y: 0. }, 5.);
            // frame.fill(&circle, Color::BLACK);
        });

        vec![geom]
    }
}
