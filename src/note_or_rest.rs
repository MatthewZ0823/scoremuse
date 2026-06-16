use iced::widget::canvas::{self};
use iced::{
    Color, Point, Vector,
    widget::canvas::{Frame, Path},
};

use crate::colors::HIGHLIGHT_COLOR;
use crate::font::{self, FontMeta};
use crate::pitch::{Pitch, PitchClass};

#[derive(Debug)]
pub struct NoteOrRest {
    /// When pitch is None its a rest
    pitch: Option<Pitch>,
    /// 1 -> Whole Note, 2 -> Half Note, 3 - Quarter Note, ...
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

pub struct NoteOrRestEl {
    note_or_rest: NoteOrRest,
    // X-Coordinate of the glyph's X=0, in staff space
    x: f32,
    // The width of the glyph in staff space
    width: f32,
}

pub enum NoteInteraction {
    None,
    // Hovering this note
    Hovering,
    // Selected this note and mouse is hovering pitch
    Selected(Pitch),
}

impl NoteOrRestEl {
    pub fn new(note_or_rest: NoteOrRest, x: f32, font: &font::FontMeta) -> Self {
        let width = Self::compute_width(&note_or_rest, font);

        Self {
            note_or_rest,
            width,
            x,
        }
    }

    fn compute_width(note_or_rest: &NoteOrRest, font: &font::FontMeta) -> f32 {
        match note_or_rest.pitch {
            None => {
                // Rest
                match note_or_rest.duration {
                    1 => font.rests_meta.whole_rest.advance_width,
                    2 => font.rests_meta.half_rest.advance_width,
                    3 => font.rests_meta.quarter_rest.advance_width,
                    _ => todo!(),
                }
            }
            Some(pitch) => {
                // Note
                let down = Self::stem_down(&pitch);
                match note_or_rest.duration {
                    1 => font.notes_meta.whole_note.advance_width,
                    2 => {
                        if down {
                            font.notes_meta.half_note.stem_down_advance_width
                        } else {
                            font.notes_meta.half_note.stem_up_advance_width
                        }
                    }
                    3 => {
                        if down {
                            font.notes_meta.quarter_note.stem_down_advance_width
                        } else {
                            font.notes_meta.quarter_note.stem_up_advance_width
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    }

    fn stem_down(pitch: &Pitch) -> bool {
        *pitch > Pitch::new(PitchClass::B, 4)
    }

    /// Gets the x-coordinate of the right bound
    pub fn get_right_bound(self: &Self) -> f32 {
        self.x + self.width
    }

    /// Gets the x-coordinate of the left bound
    pub fn get_left_bound(self: &Self) -> f32 {
        self.x
    }

    // Get the total width of the note
    pub fn get_width(self: &Self) -> f32 {
        self.width
    }

    pub fn translate_x(self: &mut Self, dx: f32) {
        self.x += dx;
    }

    // Keeps the center the same
    // Assuming only the right width might change on pitch change
    pub fn set_pitch(self: &mut Self, pitch: Pitch, font: &font::FontMeta) {
        self.note_or_rest.pitch = Some(pitch);
        self.width = Self::compute_width(&self.note_or_rest, font);
    }

    // Frame coordinates should be same as staff coordinates
    pub fn draw(
        self: &Self,
        frame: &mut Frame,
        note_interaction: &NoteInteraction,
        font: &FontMeta,
    ) {
        // Draw the "selected/hover" note
        if let NoteInteraction::Selected(hovering_pitch) = note_interaction {
            let stem_direction = if Self::stem_down(hovering_pitch) {
                StemDirection::DOWN
            } else {
                StemDirection::UP
            };

            draw_note(
                frame,
                self.note_or_rest.duration,
                Point::new(self.x, hovering_pitch.to_y_offset()),
                stem_direction,
                Some(Color::from_rgb(0.6, 0.6, 0.6)),
                &font,
            );
        }

        let color = match note_interaction {
            NoteInteraction::None => None,
            NoteInteraction::Hovering | NoteInteraction::Selected(..) => Some(HIGHLIGHT_COLOR),
        };

        match &self.note_or_rest.pitch {
            None => match self.note_or_rest.duration {
                1 => {
                    draw_glyph(
                        frame,
                        "\u{E4E3}",
                        Point::new(self.x, 0.),
                        color,
                        &font.font_iced,
                    );
                }
                2 => {
                    draw_glyph(
                        frame,
                        "\u{E4E4}",
                        Point::new(self.x, 0.),
                        color,
                        &font.font_iced,
                    );
                }
                3 => {
                    draw_glyph(
                        frame,
                        "\u{E4E5}",
                        Point::new(self.x, 0.),
                        color,
                        &font.font_iced,
                    );
                }
                _ => todo!(),
            },
            Some(pitch) => {
                let stem_direction = if Self::stem_down(pitch) {
                    StemDirection::DOWN
                } else {
                    StemDirection::UP
                };
                draw_note(
                    frame,
                    self.note_or_rest.duration,
                    Point::new(self.x, (&pitch).to_y_offset()),
                    stem_direction,
                    color,
                    &font,
                );
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum StemDirection {
    UP,
    DOWN,
}

/// Draws a note to `frame`, `frame` should be in staff coordinates
fn draw_note(
    frame: &mut Frame,
    duration: u8,
    position: Point,
    stem_direction: StemDirection,
    color: Option<Color>,
    font_meta: &FontMeta,
) {
    let notes_meta = &font_meta.notes_meta;
    let thickness = font_meta.engraving_defaults.stem_thickness;
    match duration {
        1 => {
            draw_glyph(frame, "\u{E0A2}", position, color, &font_meta.font_iced);
        }
        2 => {
            let color = draw_glyph(frame, "\u{E0A3}", position, color, &font_meta.font_iced);
            let anchor = notes_meta.half_note.stem.get_anchor(stem_direction);
            draw_stem(frame, anchor, stem_direction, thickness, color);
        }
        _ => {
            let color = draw_glyph(frame, "\u{E0A4}", position, color, &font_meta.font_iced);
            let anchor = notes_meta.quarter_note.stem.get_anchor(stem_direction);
            draw_stem(frame, anchor, stem_direction, thickness, color);
        }
    }
}

// Returns the color the glyph was drawn as
fn draw_glyph(
    frame: &mut Frame,
    glyph: &str,
    position: Point,
    color: Option<Color>,
    font: &iced::font::Font,
) -> Color {
    let glyph: canvas::Text = canvas::Text {
        font: *font,
        align_y: iced::alignment::Vertical::Center,
        position: position,
        size: 1.into(),
        ..glyph.into()
    };

    let mut _color: Color = Default::default();
    glyph.draw_with(|path, c| {
        _color = color.unwrap_or(c);
        frame.fill(&path, color.unwrap_or(c));
    });
    _color
}

fn draw_stem(
    frame: &mut Frame,
    anchor: Point,
    stem_direction: StemDirection,
    thickness: f32,
    color: Color,
) {
    let length = match stem_direction {
        StemDirection::UP => 1.,
        StemDirection::DOWN => -1.,
    };
    let stem_start = anchor;
    let stem_end = anchor + Vector::new(0., length);
    let stem_path = Path::line(stem_start, stem_end);

    let stem_stroke = canvas::Stroke {
        width: thickness,
        style: color.into(),
        ..canvas::Stroke::default()
    };
    frame.stroke(&stem_path, stem_stroke);
}
