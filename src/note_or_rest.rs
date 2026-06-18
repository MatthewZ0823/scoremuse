use iced::Size;
use iced::widget::canvas::{self};
use iced::{
    Color, Point,
    widget::canvas::{Frame, Path},
};

use crate::colors::HIGHLIGHT_COLOR;
use crate::constants::STANDARD_STAFF_SPACING;
use crate::font::{self, FontMeta, StemNoteMeta};
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
    // The advance width of the glyph in staff units
    advance_width: f32,
    right_margin: f32,
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
        let advance_width = Self::compute_advance_width(&note_or_rest, font);
        let right_margin = Self::compute_right_margin(note_or_rest.duration);

        Self {
            note_or_rest,
            advance_width,
            right_margin,
            x,
        }
    }

    fn compute_right_margin(duration: u8) -> f32 {
        (match duration {
            0 => panic!("Cannot have duration 0"),
            1 => 3.,
            2 => 2.,
            3 => 1.,
            _ => todo!(),
        }) * STANDARD_STAFF_SPACING
    }

    fn compute_advance_width(note_or_rest: &NoteOrRest, font: &font::FontMeta) -> f32 {
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

    fn stem_direction(pitch: &Pitch) -> StemDirection {
        if Self::stem_down(pitch) {
            StemDirection::DOWN
        } else {
            StemDirection::UP
        }
    }

    /// Gets the total width of the glyph, including margins
    pub fn get_width(self: &Self) -> f32 {
        self.advance_width + self.right_margin
    }

    /// Gets the x-coordinate of the right bound
    pub fn get_right_bound(self: &Self) -> f32 {
        self.x + self.get_width()
    }

    /// Gets the x-coordinate of the left bound
    pub fn get_left_bound(self: &Self) -> f32 {
        self.x
    }

    pub fn translate_x(self: &mut Self, dx: f32) {
        self.x += dx;
    }

    // Keeps the center the same
    // Assuming only the right width might change on pitch change
    pub fn set_pitch(self: &mut Self, pitch: Pitch, font: &font::FontMeta) {
        self.note_or_rest.pitch = Some(pitch);
        self.advance_width = Self::compute_advance_width(&self.note_or_rest, font);
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
            draw_note(
                frame,
                self.note_or_rest.duration,
                Point::new(self.x, hovering_pitch.to_y_offset()),
                Self::stem_direction(hovering_pitch),
                Some(Color::from_rgb(0.6, 0.6, 0.6)),
                &font,
            );
        }

        let color = match note_interaction {
            NoteInteraction::None => None,
            NoteInteraction::Hovering | NoteInteraction::Selected(..) => Some(HIGHLIGHT_COLOR),
        };

        match &self.note_or_rest.pitch {
            None => {
                let glyph_str = match self.note_or_rest.duration {
                    1 => "\u{E4E3}",
                    2 => "\u{E4E4}",
                    3 => "\u{E4E5}",
                    _ => todo!(),
                };

                draw_glyph(
                    frame,
                    glyph_str,
                    Point::new(self.x, 2. * STANDARD_STAFF_SPACING),
                    color,
                    &font.font_iced,
                );
            }
            Some(pitch) => {
                draw_note(
                    frame,
                    self.note_or_rest.duration,
                    Point::new(self.x, (&pitch).to_y_offset()),
                    Self::stem_direction(pitch),
                    color,
                    &font,
                );
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
            draw_stem(
                frame,
                &position,
                &notes_meta.half_note.stem,
                &stem_direction,
                thickness,
                &color,
            );
        }
        _ => {
            let color = draw_glyph(frame, "\u{E0A4}", position, color, &font_meta.font_iced);
            draw_stem(
                frame,
                &position,
                &notes_meta.quarter_note.stem,
                &stem_direction,
                thickness,
                &color,
            );
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
        size: (4. * STANDARD_STAFF_SPACING).into(),
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
    note_position: &Point,
    stem_meta: &StemNoteMeta,
    stem_direction: &StemDirection,
    thickness: f32,
    color: &Color,
) {
    let anchor = *note_position + stem_meta.get_anchor(stem_direction);
    let sign = match stem_direction {
        StemDirection::UP => -1.,
        StemDirection::DOWN => 1.,
    };
    let length = stem_meta.get_length(stem_direction) * sign;
    let width = thickness * sign;

    let stem = Path::rectangle(anchor, Size::new(width, length));
    frame.fill(&stem, *color);
}
