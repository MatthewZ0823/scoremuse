use std::time::Duration;

use iced::widget::canvas::{self};
use iced::{
    Color, Point,
    widget::canvas::{Frame, Path},
};
use iced::{Size, Vector};
use num_traits::Pow;

use crate::colors::{HIGHLIGHT_COLOR, HOVER_COLOR};
use crate::constants::{ACCIDENTAL_SPACING, BPM, STANDARD_STAFF_SPACING};
use crate::font::{self, FontMeta, HasStem};
use crate::pitch::{Accidental, Pitch, PitchClass};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
/// 0 -> Whole Note, 1 -> Half Note, 2 - Quarter Note, ...
/// Durations shorter than 128th notes note yet implemented
pub struct BaseDuration(pub u8);

impl BaseDuration {
    /// Gets the duration of the note in beats
    pub fn get_duration_beats(&self) -> f32 {
        2_f32.pow(2. - self.0 as f32)
    }

    /// Construct a base duration from a number of beats
    ///
    /// TODO: Make work for different time signatures
    /// TODO: Handle not clean number of beats, ie dotted notes
    pub fn from_duration_beats(beats: f32) -> Self {
        Self((2 - beats.log2() as i32) as u8)
    }
}

impl PartialOrd for BaseDuration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BaseDuration {
    // Order is flippde
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

#[derive(Debug)]
pub struct NoteOrRest {
    /// When pitch is None its a rest
    pitch: Option<Pitch>,
    base_duration: BaseDuration,
}

impl NoteOrRest {
    pub fn new(pitch: Option<Pitch>, base_duration: BaseDuration) -> Self {
        NoteOrRest {
            pitch: pitch,
            base_duration,
        }
    }

    pub fn get_base_duration(&self) -> BaseDuration {
        self.base_duration
    }
}

#[derive(Debug)]
pub struct NoteOrRestEl {
    note_or_rest: NoteOrRest,
    /// X-Coordinate of the note/rest's X=0, in staff space
    x: f32,
    /// Width of the note/rest excluding margins, in staff units
    width: f32,
    /// Right margin of the note/rest, in staff units
    right_margin: f32,
}

pub enum NoteInteraction {
    None,
    Hovering,
    Selected,
    Dragging,
}

impl NoteOrRestEl {
    pub fn new(note_or_rest: NoteOrRest, x: f32, font: &font::FontMeta) -> Self {
        let width = Self::compute_width(&note_or_rest, font);
        let right_margin = Self::compute_right_margin(&note_or_rest.base_duration);

        Self {
            note_or_rest,
            x,
            width,
            right_margin,
        }
    }

    fn compute_right_margin(base_duration: &BaseDuration) -> f32 {
        (match base_duration.0 {
            0 => 4.,
            1 => 3.,
            2 => 2.,
            3 => 1.,
            _ => 0.5, // TODO: Adjust spacing
        }) * STANDARD_STAFF_SPACING
    }

    fn compute_width(note_or_rest: &NoteOrRest, font: &font::FontMeta) -> f32 {
        match note_or_rest.pitch {
            None => {
                // Rest
                font.rests_meta.rests[note_or_rest.base_duration.0 as usize].advance_width
            }
            Some(pitch) => {
                // Note
                let stem_dir = Self::stem_direction(&pitch);
                let note_advance_width = font
                    .notes_meta
                    .get_advance_width(&note_or_rest.base_duration, &stem_dir);

                let accidental_width = pitch.accidental.map_or(0., |accidental| {
                    font.accidentals_meta.get_advance_width(accidental)
                }) + ACCIDENTAL_SPACING;

                note_advance_width + accidental_width
            }
        }
    }

    fn stem_down(pitch: &Pitch) -> bool {
        pitch.to_staff_index().0 > Pitch::new(PitchClass::B, 4, None).to_staff_index().0
    }

    fn stem_direction(pitch: &Pitch) -> StemDirection {
        if Self::stem_down(pitch) {
            StemDirection::DOWN
        } else {
            StemDirection::UP
        }
    }

    /// Get the duration of this note in beats
    pub fn get_duration_beats(&self) -> f32 {
        self.note_or_rest.get_base_duration().get_duration_beats()
    }

    pub fn get_base_duration(&self) -> BaseDuration {
        self.note_or_rest.get_base_duration()
    }

    /// How long it would take to play the bar
    pub fn get_time_duration(&self) -> Duration {
        Duration::from_secs_f32(self.get_duration_beats() / BPM as f32 * 60.)
    }

    /// Gets the total width of the glyph, including margins
    pub fn get_total_width(self: &Self) -> f32 {
        self.width + self.right_margin
    }

    /// Gets the x-coordinate of the right bound
    pub fn get_right_bound(self: &Self) -> f32 {
        self.x + self.get_total_width()
    }

    /// Gets the x-coordinate of the left bound
    pub fn get_left_bound(self: &Self) -> f32 {
        self.x
    }

    pub fn set_x(self: &mut Self, x: f32) {
        self.x = x;
    }

    pub fn translate_x(self: &mut Self, dx: f32) {
        self.x += dx;
    }

    pub fn get_pitch(&self) -> Option<Pitch> {
        self.note_or_rest.pitch
    }

    /// Setting pitch to None turns the note into a rest
    /// May change width
    pub fn set_pitch(self: &mut Self, pitch: Option<Pitch>, font: &FontMeta) {
        self.note_or_rest.pitch = pitch;
        self.fix_width(font);
    }

    /// May change width and right margin
    pub fn set_base_duration(&mut self, base_duration: BaseDuration, font: &FontMeta) {
        self.note_or_rest.base_duration = base_duration;
        self.fix_width(font);
    }

    /// Fix the width/right margin after a change in self
    fn fix_width(&mut self, font: &FontMeta) {
        self.width = Self::compute_width(&self.note_or_rest, font);
        self.right_margin = Self::compute_right_margin(&self.note_or_rest.base_duration);
    }

    // Frame coordinates should be same as staff coordinates
    pub fn draw(
        self: &Self,
        frame: &mut Frame,
        note_interaction: &NoteInteraction,
        font: &FontMeta,
    ) {
        let color = match note_interaction {
            NoteInteraction::None => None,
            NoteInteraction::Hovering => Some(HOVER_COLOR),
            NoteInteraction::Selected | NoteInteraction::Dragging => Some(HIGHLIGHT_COLOR),
        };

        match &self.note_or_rest.pitch {
            None => {
                let glyph_str = match self.note_or_rest.base_duration.0 {
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

                let y = if self.note_or_rest.base_duration.0 == 0 {
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
                let mut pos = Point::new(self.x, (&pitch).to_y());

                pitch.accidental.map(|a| {
                    draw_glyph(frame, a.to_glyph(), pos, color, &font.font_iced);
                    pos += Vector::new(
                        font.accidentals_meta.get_advance_width(a) + ACCIDENTAL_SPACING,
                        0.,
                    );
                });

                draw_note(
                    frame,
                    self.note_or_rest.base_duration,
                    pos,
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
    base_duration: BaseDuration,
    position: Point,
    stem_direction: StemDirection,
    color: Option<Color>,
    font_meta: &FontMeta,
) {
    if base_duration.0 > 7 {
        panic!("Notes with base duration less than 128th not yet implemented");
    }

    let notes_meta = &font_meta.notes_meta;
    let stem_thickness = font_meta.engraving_defaults.stem_thickness;

    let note_head_glyph = match base_duration.0 {
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

    if base_duration.0 != 0 {
        let stem_meta = match base_duration.0 {
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
            stem_thickness,
            &note_head_color,
            base_duration,
            font_meta,
        );
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
    stem_thickness: f32,
    color: &Color,
    base_duration: BaseDuration,
    font_meta: &FontMeta,
) {
    let anchor = *note_position + stem_meta.get_stem_anchor(stem_direction);
    let sign = match stem_direction {
        StemDirection::UP => -1.,
        StemDirection::DOWN => 1.,
    };
    let length = stem_meta.get_length(stem_direction) * sign;
    let width = stem_thickness * sign;

    let stem = Path::rectangle(anchor, Size::new(width, length));
    frame.fill(&stem, *color);

    if base_duration.0 > 2 {
        let stem_end = anchor
            + Vector::new(
                match stem_direction {
                    StemDirection::UP => -stem_thickness,
                    StemDirection::DOWN => 0.,
                },
                length,
            );
        draw_flag(
            frame,
            &stem_end,
            &stem_direction,
            &color,
            base_duration,
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
    base_duration: BaseDuration,
    font: &iced::Font,
) {
    if base_duration.0 <= 2 || base_duration.0 > 7 {
        panic!("No flag for note with this duration");
    }

    let glyph_unicode: u32 = 57920
        + 2 * (base_duration.0 as u32 - 3)
        + (match stem_direction {
            StemDirection::UP => 0,
            StemDirection::DOWN => 1,
        });
    let glyph = char::from_u32(glyph_unicode).unwrap().to_string();
    draw_glyph(frame, &glyph, *stem_end, Some(*color), font);
}
