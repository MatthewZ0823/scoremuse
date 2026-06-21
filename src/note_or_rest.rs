use iced::widget::canvas::{self};
use iced::{
    Color, Point,
    widget::canvas::{Frame, Path},
};
use iced::{Size, Vector};

use crate::colors::HIGHLIGHT_COLOR;
use crate::constants::STANDARD_STAFF_SPACING;
use crate::font::{self, FontMeta, GetAdvanceWidth, HasStem};
use crate::pitch::{Pitch, PitchClass};

#[derive(Debug)]
pub struct NoteOrRest {
    /// When pitch is None its a rest
    pitch: Option<Pitch>,
    /// 0 -> Whole Note, 1 -> Half Note, 2 - Quarter Note, ...
    /// Durations shorter than 128th notes note yet implemented
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
            0 => 4.,
            1 => 3.,
            2 => 2.,
            3 => 1.,
            _ => 0.5, // TODO: Adjust spacing
        }) * STANDARD_STAFF_SPACING
    }

    fn compute_advance_width(note_or_rest: &NoteOrRest, font: &font::FontMeta) -> f32 {
        match note_or_rest.pitch {
            None => {
                // Rest
                font.rests_meta.rests[note_or_rest.duration as usize].advance_width
            }
            Some(pitch) => {
                // Note
                let stem_dir = Self::stem_direction(&pitch);
                let get_advance_width: Box<&dyn GetAdvanceWidth> = match note_or_rest.duration {
                    0 => Box::new(&font.notes_meta.whole_note),
                    1 => Box::new(&font.notes_meta.half_note),
                    2 => Box::new(&font.notes_meta.quarter_note),
                    3 => Box::new(&font.notes_meta.note_8th),
                    4 => Box::new(&font.notes_meta.note_16th),
                    5 => Box::new(&font.notes_meta.note_32nd),
                    6 => Box::new(&font.notes_meta.note_64th),
                    7 => Box::new(&font.notes_meta.note_128th),
                    _ => panic!("Notes shorter than 128th have not been implemented"),
                };

                get_advance_width.get_advance_width(&stem_dir)
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
                    0 => "\u{E4E3}",
                    1 => "\u{E4E4}",
                    2 => "\u{E4E5}",
                    3 => "\u{E4E6}",
                    4 => "\u{E4E7}",
                    5 => "\u{E4E8}",
                    6 => "\u{E4E9}",
                    7 => "\u{E4EA}",
                    _ => panic!("Rests shorter than 128th not yet implemented"),
                };

                let y = if self.note_or_rest.duration == 0 {
                    STANDARD_STAFF_SPACING
                } else {
                    2. * STANDARD_STAFF_SPACING
                };

                draw_glyph(
                    frame,
                    glyph_str,
                    Point::new(self.x, y),
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
    if duration > 7 {
        panic!("Notes with duration less than 128th not yet implemented");
    }

    let notes_meta = &font_meta.notes_meta;
    let thickness = font_meta.engraving_defaults.stem_thickness;

    let note_head_glyph = match duration {
        0 => "\u{E0A2}",
        1 => "\u{E0A3}",
        _ => "\u{E0A4}",
    };
    let note_head_color = draw_glyph(
        frame,
        note_head_glyph,
        position,
        color,
        &font_meta.font_iced,
    );

    match duration {
        0 => (),
        _ => {
            let stem_meta = match duration {
                1 => &notes_meta.half_note,
                2 => &notes_meta.quarter_note,
                3 => &notes_meta.note_8th,
                4 => &notes_meta.note_16th,
                5 => &notes_meta.note_32nd,
                6 => &notes_meta.note_64th,
                7 => &notes_meta.note_128th,
                _ => panic!(),
            };
            draw_stem(
                frame,
                &position,
                stem_meta,
                &stem_direction,
                thickness,
                &note_head_color,
                duration,
                font_meta,
            );
        }
    };
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

/// Draws flags also
fn draw_stem(
    frame: &mut Frame,
    note_position: &Point,
    stem_meta: &impl HasStem,
    stem_direction: &StemDirection,
    thickness: f32,
    color: &Color,
    duration: u8,
    font_meta: &FontMeta,
) {
    let anchor = *note_position + stem_meta.get_stem_anchor(stem_direction);
    let sign = match stem_direction {
        StemDirection::UP => -1.,
        StemDirection::DOWN => 1.,
    };
    let length = stem_meta.get_length(stem_direction) * sign;
    let width = thickness * sign;

    let stem = Path::rectangle(anchor, Size::new(width, length));
    frame.fill(&stem, *color);

    if duration > 2 {
        let stem_end = anchor
            + Vector::new(
                match stem_direction {
                    StemDirection::UP => -thickness,
                    StemDirection::DOWN => 0.,
                },
                length,
            );
        draw_flag(
            frame,
            &stem_end,
            &stem_direction,
            &color,
            duration,
            &font_meta.font_iced,
        );
    }
}

/// `stem_end` is the left side of the note's stem end
fn draw_flag(
    frame: &mut Frame,
    stem_end: &Point,
    stem_direction: &StemDirection,
    color: &Color,
    duration: u8,
    font: &iced::Font,
) {
    if duration <= 2 || duration > 7 {
        panic!("No flag for note with this duration");
    }

    let glyph_unicode: u32 = 57920
        + 2 * (duration as u32 - 3)
        + (match stem_direction {
            StemDirection::UP => 0,
            StemDirection::DOWN => 1,
        });
    let glyph = char::from_u32(glyph_unicode).unwrap().to_string();
    draw_glyph(frame, &glyph, *stem_end, Some(*color), font);
}
