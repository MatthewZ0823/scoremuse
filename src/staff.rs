use crate::{
    canvas_el::{CWidgetUpdate, CanvasWidget},
    canvas_svg::{CanvasSVG, Positioning::*, SizingMode::*},
    staff,
};
use core::f32;
use iced::{
    Point, Rectangle, Renderer, Size, mouse,
    widget::canvas::{self},
};
use std::array::from_fn;

const BARLINE_Y_SPACING: f32 = 20.;
const NOTE_Y_SPACING: f32 = BARLINE_Y_SPACING / 2.;

const TREBLE_CLEF_ASPECT_RATIO: f32 = 95.116 / 153.12;
const TREBLE_CLEF_PATH: &str = "src/assets/treble_clef.svg";

const FILLED_NOTE_HEAD_ASPECT_RATIO: f32 = 260. / 200.;
const FILLED_NOTE_HEAD_PATH: &str = "src/assets/filled_note_head.svg";

#[derive(Debug, Default)]
pub struct StaffState {
    bar_lines: Option<[canvas::Path; 5]>,
    hovering: Option<Pitch>,
}

pub struct StaffWidget<'a> {
    pub notes: &'a [Pitch],
}

// Outgoing messages from Staff
pub enum Message {
    AddNote(Pitch),
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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

impl<'a> CanvasWidget for StaffWidget<'a> {
    type State = StaffState;
    type Message = Message;

    fn draw_fn(&self, state: &StaffState, size: &Size, frame: &mut canvas::Frame<Renderer>) {
        let bar_lines: &[canvas::Path; 5] = match &state.bar_lines {
            Some(bar_lines) => bar_lines,
            None => &create_bar_lines(size.width),
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
    }

    fn update_fn(
        &self,
        state: &mut StaffState,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<CWidgetUpdate<staff::Message>> {
        // TODO: Clean up update logic
        match event {
            iced::Event::Window(
                iced::window::Event::Opened { .. }
                | iced::window::Event::Resized(_)
                | iced::window::Event::Rescaled(_),
            ) => state.bar_lines = Some(create_bar_lines(bounds.width)),
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

                    Some(CWidgetUpdate::REDRAW)
                }
                mouse::Event::CursorLeft => {
                    state.hovering = None;
                    Some(CWidgetUpdate::REDRAW)
                }
                mouse::Event::ButtonPressed(button) => {
                    if *button == mouse::Button::Left {
                        match state.hovering {
                            Some(hovered) => {
                                Some(CWidgetUpdate::MESSAGE(Message::AddNote(hovered)))
                            }
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

fn create_bar_lines(width: f32) -> [canvas::Path; 5] {
    from_fn(|i| {
        let y = i as f32 * BARLINE_Y_SPACING;
        let from = Point::new(0., y);
        let to = Point::new(width, y);

        canvas::Path::line(from, to)
    })
}

// impl canvas::Program<Message> for Staff {
//     type State = StaffInternal;
//
//     fn draw(
//         &self,
//         state: &Self::State,
//         renderer: &Renderer,
//         _theme: &Theme,
//         bounds: Rectangle,
//         _cursor: mouse::Cursor,
//     ) -> Vec<canvas::Geometry<Renderer>> {
//         let geom = self.cache.draw(renderer, bounds.size(), |frame| {
//             let bar_lines: &[canvas::Path; 5] = match &state.bar_lines {
//                 Some(bar_lines) => bar_lines,
//                 None => &create_bar_lines(&bounds),
//             };
//
//             bar_lines.iter().for_each(|path| {
//                 frame.stroke(&path, canvas::Stroke::default());
//             });
//
//             let treble_clef = CanvasSVG::new(
//                 TREBLE_CLEF_PATH,
//                 TREBLE_CLEF_ASPECT_RATIO,
//                 TopLeft(Point::new(0., -1.65 * BARLINE_Y_SPACING)),
//                 HeightOnly(7.5 * BARLINE_Y_SPACING),
//             );
//             treble_clef.draw_to_frame(frame);
//
//             let note = |pitch| {
//                 CanvasSVG::new(
//                     FILLED_NOTE_HEAD_PATH,
//                     FILLED_NOTE_HEAD_ASPECT_RATIO,
//                     Centered(Point::new(100., pitch_to_y_offset(pitch))),
//                     HeightOnly(20.),
//                 )
//             };
//             self.notes.iter().for_each(|pitch| {
//                 note(pitch).draw_to_frame(frame);
//             });
//
//             match state.hovering {
//                 Some(hovered) => {
//                     note(&hovered).draw_to_frame(frame);
//                 }
//                 None => (),
//             }
//         });
//
//         vec![geom]
//     }
// }
