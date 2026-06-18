#![allow(dead_code)]

use serde::Deserialize;
use serde_json;
use std::{fmt, fs, io, path::PathBuf};

use iced::{Font, Task, Vector, font};

use crate::{constants::STANDARD_STAFF_SPACING, note_or_rest::StemDirection};

#[derive(Debug, Clone)]
pub enum Error {
    IO(io::ErrorKind),
    Deserialize,
    FontLoadError(iced::font::Error),
}

// font_name is where to find the font
pub fn load_font(font_name: String) -> Task<Result<FontMeta, Error>> {
    const FONT_DIR: &str = "fonts";

    get_bytes(
        &PathBuf::from(FONT_DIR)
            .join(&font_name)
            .join(format!("{font_name}.otf")),
    )
    .and_then(move |font_bytes| {
        let font_name = font_name.clone();
        font::load(font_bytes)
            .map_err(|e| Error::FontLoadError(e))
            .then(move |_| {
                get_bytes(
                    &PathBuf::from(FONT_DIR)
                        .join(font_name.clone())
                        .join("metadata.json"),
                )
            })
            .and_then(|meta_bytes| deserialize_meta(meta_bytes))
    })
}

fn deserialize_meta(bytes: Vec<u8>) -> Task<Result<FontMeta, Error>> {
    Task::done(serde_json::from_slice(&bytes).map_err(|_| Error::Deserialize))
}

fn get_bytes(path: &PathBuf) -> Task<Result<Vec<u8>, Error>> {
    match fs::read(path) {
        Ok(b) => Task::done(Ok(b)),
        Err(e) => Task::done(Err(Error::IO(e.kind()))),
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "RawFontMeta")]
/// The parsed [SMuFL](https://w3c-cg.github.io/smufl/latest/index.html) font metadata
///
/// Should all be in Staff units, as described in notes.md, unless otherwise stated
pub struct FontMeta {
    /// The name of the font, as read from the metadata.json
    pub font_name: String,
    /// The font as an iced struct
    pub font_iced: iced::Font,
    /// The engraving defaults, expressed in Staff units
    pub engraving_defaults: EngravingDefaults,
    /// Metadata pretaining to the notes
    pub notes_meta: NotesMeta,
    /// Metadata pretaining to the rests
    pub rests_meta: RestsMeta,
    /// Metadata pretaining to the barlines
    pub barlines_meta: BarlinesMeta,
}

#[derive(Clone, Debug)]
pub struct BarlinesMeta {
    pub single_advance_width: f32,
    pub thin_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct NotesMeta {
    pub quarter_note: QuarterNoteMeta,
    pub half_note: HalfNoteMeta,
    pub whole_note: WholeNoteMeta,
}

#[derive(Clone, Debug)]
pub struct RestsMeta {
    pub quarter_rest: RestMeta,
    pub half_rest: RestMeta,
    pub whole_rest: RestMeta,
}

#[derive(Clone, Debug)]
pub struct RestMeta {
    pub advance_width: f32,
}

#[derive(Clone, Debug)]
pub struct EngravingDefaults {
    /// The thickness of the staff lines
    pub staff_line_thickness: f32,
    /// The thickness of a stem
    pub stem_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct QuarterNoteMeta {
    pub stem_up_advance_width: f32,
    pub stem_down_advance_width: f32,
    pub stem: StemNoteMeta,
}

#[derive(Clone, Debug)]
pub struct HalfNoteMeta {
    pub stem_up_advance_width: f32,
    pub stem_down_advance_width: f32,
    pub stem: StemNoteMeta,
}

#[derive(Clone, Debug)]
pub struct WholeNoteMeta {
    pub advance_width: f32,
}

#[derive(Clone, Debug)]
// Metadata for notes with stems
pub struct StemNoteMeta {
    pub stem_up_se: Vector,
    pub stem_down_nw: Vector,
}

impl StemNoteMeta {
    pub fn get_anchor(&self, stem_direction: &StemDirection) -> Vector {
        match stem_direction {
            StemDirection::UP => self.stem_up_se,
            StemDirection::DOWN => self.stem_down_nw,
        }
    }

    /// Get the (unsigned) length of the stem if there are no modifications to the length
    pub fn get_length(&self, stem_direction: &StemDirection) -> f32 {
        let anchor = self.get_anchor(stem_direction);
        let sign = match stem_direction {
            StemDirection::UP => 1.,
            StemDirection::DOWN => -1.,
        };
        3.5 * STANDARD_STAFF_SPACING + sign * anchor.y
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFontMeta {
    font_name: String,
    engraving_defaults: RawEngravingDefaults,
    glyph_advance_widths: RawGlyphAdvanceWidths,
    glyphs_with_anchors: RawGlyphsWithAnchors,
}

impl TryFrom<RawFontMeta> for FontMeta {
    type Error = DeserializeError;

    fn try_from(value: RawFontMeta) -> Result<Self, Self::Error> {
        let font_name: &'static str = value.font_name.clone().leak();
        let font_iced = Font::with_name(font_name);

        Ok(FontMeta {
            font_iced,
            font_name: value.font_name,
            barlines_meta: BarlinesMeta {
                single_advance_width: value.glyph_advance_widths.barline_single
                    * STANDARD_STAFF_SPACING,
                thin_thickness: value.engraving_defaults.thin_barline_thickness
                    * STANDARD_STAFF_SPACING,
            },
            engraving_defaults: EngravingDefaults {
                staff_line_thickness: value.engraving_defaults.staff_line_thickness
                    * STANDARD_STAFF_SPACING,
                stem_thickness: value.engraving_defaults.stem_thickness * STANDARD_STAFF_SPACING,
            },
            notes_meta: NotesMeta {
                quarter_note: QuarterNoteMeta {
                    stem: StemNoteMeta::try_from(value.glyphs_with_anchors.notehead_black)?,
                    stem_up_advance_width: value.glyph_advance_widths.note_quarter_up
                        * STANDARD_STAFF_SPACING,
                    stem_down_advance_width: value.glyph_advance_widths.note_quarter_down
                        * STANDARD_STAFF_SPACING,
                },
                half_note: HalfNoteMeta {
                    stem: StemNoteMeta::try_from(value.glyphs_with_anchors.notehead_half)?,
                    stem_up_advance_width: value.glyph_advance_widths.note_half_up
                        * STANDARD_STAFF_SPACING,
                    stem_down_advance_width: value.glyph_advance_widths.note_half_down
                        * STANDARD_STAFF_SPACING,
                },
                whole_note: WholeNoteMeta {
                    advance_width: value.glyph_advance_widths.note_whole * STANDARD_STAFF_SPACING,
                },
            },
            rests_meta: RestsMeta {
                quarter_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_quarter * STANDARD_STAFF_SPACING,
                },
                half_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_half * STANDARD_STAFF_SPACING,
                },
                whole_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_whole * STANDARD_STAFF_SPACING,
                },
            },
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEngravingDefaults {
    staff_line_thickness: f32,
    stem_thickness: f32,
    thin_barline_thickness: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGlyphAdvanceWidths {
    note_whole: f32,
    note_half_up: f32,
    note_half_down: f32,
    note_quarter_up: f32,
    note_quarter_down: f32,
    note_8th_up: f32,
    note_8th_down: f32,
    note_16th_up: f32,
    note_16th_down: f32,
    note_32nd_up: f32,
    note_32nd_down: f32,
    rest_whole: f32,
    rest_half: f32,
    rest_quarter: f32,
    barline_single: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGlyphsWithAnchors {
    notehead_whole: RawAnchors,
    notehead_half: RawAnchors,
    notehead_black: RawAnchors,
}

#[derive(Clone, Debug, Deserialize)]
struct RawAnchors {
    #[serde(rename = "stemUpNW")]
    stem_up_nw: Option<(f32, f32)>,
    #[serde(rename = "stemUpSE")]
    stem_up_se: Option<(f32, f32)>,
    #[serde(rename = "stemDownNW")]
    stem_down_nw: Option<(f32, f32)>,
    #[serde(rename = "stemDownSW")]
    stem_down_sw: Option<(f32, f32)>,
}

impl TryFrom<RawAnchors> for StemNoteMeta {
    type Error = DeserializeError;

    fn try_from(value: RawAnchors) -> Result<Self, Self::Error> {
        let stem_up_se = value
            .stem_up_se
            .map(|v| Vector::new(v.0 * STANDARD_STAFF_SPACING, v.1 * -STANDARD_STAFF_SPACING))
            .ok_or(DeserializeError)?;
        let stem_down_nw = value
            .stem_down_nw
            .map(|v| Vector::new(v.0 * STANDARD_STAFF_SPACING, v.1 * -STANDARD_STAFF_SPACING))
            .ok_or(DeserializeError)?;

        Ok(Self {
            stem_up_se,
            stem_down_nw,
        })
    }
}

// TODO: Better error handling
#[derive(Debug)]
struct DeserializeError;
impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deserialize Error")
    }
}
