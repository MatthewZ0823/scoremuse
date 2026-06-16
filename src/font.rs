#![allow(dead_code)]

use serde::Deserialize;
use serde_json;
use std::{fmt, fs, io, path::PathBuf};

use iced::{Font, Point, Task, font};

use crate::note_or_rest::StemDirection;

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
pub struct FontMeta {
    pub font_name: String,
    pub font_iced: iced::Font,
    pub engraving_defaults: EngravingDefaults,
    pub notes_meta: NotesMeta,
    pub rests_meta: RestsMeta,
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
    pub staff_line_thickness: f32,
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
    pub stem_up_se: Point,
    pub stem_down_nw: Point,
}

impl StemNoteMeta {
    pub fn get_anchor(&self, stem_direction: StemDirection) -> Point {
        match stem_direction {
            StemDirection::UP => self.stem_up_se,
            StemDirection::DOWN => self.stem_down_nw,
        }
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
            engraving_defaults: EngravingDefaults {
                staff_line_thickness: value.engraving_defaults.staff_line_thickness,
                stem_thickness: value.engraving_defaults.stem_thickness,
            },
            notes_meta: NotesMeta {
                quarter_note: QuarterNoteMeta {
                    stem: StemNoteMeta::try_from(value.glyphs_with_anchors.notehead_black)?,
                    stem_up_advance_width: value.glyph_advance_widths.note_quarter_up,
                    stem_down_advance_width: value.glyph_advance_widths.note_quarter_down,
                },
                half_note: HalfNoteMeta {
                    stem: StemNoteMeta::try_from(value.glyphs_with_anchors.notehead_half)?,
                    stem_up_advance_width: value.glyph_advance_widths.note_half_up,
                    stem_down_advance_width: value.glyph_advance_widths.note_half_down,
                },
                whole_note: WholeNoteMeta {
                    advance_width: value.glyph_advance_widths.note_whole,
                },
            },
            rests_meta: RestsMeta {
                quarter_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_quarter,
                },
                half_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_half,
                },
                whole_rest: RestMeta {
                    advance_width: value.glyph_advance_widths.rest_whole,
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
        let stem_up_se = value.stem_up_se.map(|v| v.into()).ok_or(DeserializeError)?;
        let stem_down_nw = value
            .stem_down_nw
            .map(|v| v.into())
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
