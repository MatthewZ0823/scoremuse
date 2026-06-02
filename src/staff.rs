use core::f32;
use std::array::from_fn;
use std::rc::Rc;

use crate::canvas_svg::{CanvasSVG, Positioning::*, SizingMode::*};
use crate::note::{StemDirection, draw_note, draw_quarter_rest, draw_rect_rest};
use iced::widget::Action;
use iced::widget::canvas::{self, Frame, Path};
use iced::{Point, Rectangle, Renderer, Theme, Vector, mouse};

use crate::Message;

const BARLINE_Y_SPACING: f32 = 15.;
const NOTE_Y_SPACING: f32 = BARLINE_Y_SPACING / 2.;

const TREBLE_CLEF_ASPECT_RATIO: f32 = 95.116 / 153.12;
const TREBLE_CLEF_PATH: &str = "src/assets/treble_clef.svg";

#[derive(Debug, Default)]
pub struct Staff {
    pub bars: Vec<Bar>,
}

#[derive(Default)]
pub struct State {
    staff_cache: StaffCache,
    bar_lines: Option<[canvas::Path; 5]>,
    hovering: Option<Pitch>,
}

#[derive(Default)]
struct StaffCache {
    cache: canvas::Cache,
    bar_caches: Vec<BarCache>,
}

struct BarCache {
    width: f32,
    bar: Rc<Bar>,
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

#[derive(Debug)]
pub struct NoteOrRest {
    // When pitch is None its a rest
    pitch: Option<Pitch>,
    // 1 -> Whole Note, 2 -> Half Note, 3 - Quarter Note, ...
    duration: u8,
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
}

impl Bar {
    pub fn new(notes: Vec<NoteOrRest>) -> Self {
        Bar { notes: notes }
    }
}

/// Returns the width of the rendered note
fn render_note(x: f32, note: &NoteOrRest, frame: &mut Frame) -> f32 {
    let mut stem_down = false;

    match note.pitch {
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
                stem_down = true;
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

    (match note.duration {
        1 => 8.,
        2 => 4.,
        3 => 3.,
        _ => {
            if stem_down {
                2.
            } else {
                3.
            }
        }
    }) * BARLINE_Y_SPACING
}

// Barlines not included
// Returns the width of the rendered bar
fn render_bar(x: f32, bar: &Bar, frame: &mut Frame) -> f32 {
    let mut x_ = x;
    for note in bar.notes.iter() {
        let w = render_note(x_, note, frame);
        x_ += w;
    }

    let barline_path = Path::line(Point::new(x_, 0.), Point::new(x_, 4. * BARLINE_Y_SPACING));
    frame.stroke(&barline_path, canvas::Stroke::default());

    x_ - x
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
    type State = State;

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
            // TODO: This frame translation down kinda sucks, think about a way that scales to
            // multiple staffs
            frame.translate(Vector::new(0., 30.));
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

            // How much space to give after a bar
            const BAR_PADDING: f32 = 2. * BARLINE_Y_SPACING;
            {
                let mut x = 100.;
                for bar in self.bars.iter() {
                    let width = render_bar(x, &bar, frame);
                    x += width + BAR_PADDING;
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
