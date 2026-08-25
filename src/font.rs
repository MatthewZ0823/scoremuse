#![allow(dead_code)]

use serde::Deserialize;
use serde_json;
use std::{array::from_fn, fmt, fs, io, path::PathBuf};

use iced::{Font, Task, Vector, font};

use crate::{
    constants::STANDARD_STAFF_SPACING,
    note_or_rest::{BaseDuration, StemDirection},
    pitch::Accidental,
};

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
    /// Metadata pretaining to accidentals
    pub accidentals_meta: AccidentalsMeta,
}

#[derive(Clone, Debug)]
pub struct AccidentalsMeta {
    pub sharp_advance_width: f32,
    pub natural_advance_width: f32,
    pub flat_advance_width: f32,
}

impl AccidentalsMeta {
    pub fn get_advance_width(&self, accidental: Accidental) -> f32 {
        match accidental {
            Accidental::Sharp => self.sharp_advance_width,
            Accidental::Natural => self.natural_advance_width,
            Accidental::Flat => self.flat_advance_width,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BarlinesMeta {
    pub single_advance_width: f32,
    pub thin_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct NotesMeta {
    pub whole_note: WholeNoteMeta,
    pub half_note: StemNoteMeta,
    pub quarter_note: StemNoteMeta,
    pub note_8th: StemNoteMeta,
    pub note_16th: StemNoteMeta,
    pub note_32nd: StemNoteMeta,
    pub note_64th: StemNoteMeta,
    pub note_128th: StemNoteMeta,
}

impl NotesMeta {
    /// Get the advance width of a note with `base_duration` and `stem_direction` in this font
    pub fn get_advance_width(
        &self,
        base_duration: &BaseDuration,
        stem_direction: &StemDirection,
    ) -> f32 {
        match base_duration.0 {
            0 => self.whole_note.get_advance_width(&stem_direction),
            1 => self.half_note.get_advance_width(&stem_direction),
            2 => self.quarter_note.get_advance_width(&stem_direction),
            3 => self.note_8th.get_advance_width(&stem_direction),
            4 => self.note_16th.get_advance_width(&stem_direction),
            5 => self.note_32nd.get_advance_width(&stem_direction),
            6 => self.note_64th.get_advance_width(&stem_direction),
            7 => self.note_128th.get_advance_width(&stem_direction),
            _ => panic!("Notes shorter than 128th have not been implemented"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RestsMeta {
    /// `rests[base_duration.0]` is the metadata for the rest with `base_duration`
    pub rests: [RestMeta; 8],
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
/// Metadata for notes with stems
pub struct StemNoteMeta {
    pub stem_up_advance_width: f32,
    pub stem_down_advance_width: f32,
    pub stem_anchors: StemAnchors,
}

impl StemNoteMeta {
    /// converts `stem_up_advance_width_smufl` and `stem_down_advance_width_smufl` into staff units
    /// leaves `stem_anchors` untouched
    fn from_smufl_units(
        stem_up_advance_width_smufl: f32,
        stem_down_advance_width_smufl: f32,
        stem_anchors: StemAnchors,
    ) -> Self {
        Self {
            stem_up_advance_width: stem_up_advance_width_smufl * STANDARD_STAFF_SPACING,
            stem_down_advance_width: stem_down_advance_width_smufl * STANDARD_STAFF_SPACING,
            stem_anchors,
        }
    }
}

#[derive(Clone, Debug)]
/// Position of the stem anchors relative to the origin of the note
pub struct StemAnchors {
    pub stem_up_se: Vector,
    pub stem_down_nw: Vector,
}

#[derive(Clone, Debug)]
pub struct WholeNoteMeta {
    pub advance_width: f32,
}

pub trait HasStem {
    /// Position of the stem anchor on the notehead relative to the origin of the note
    fn get_stem_anchor(&self, stem_direction: &StemDirection) -> Vector;

    /// Get the length of the stem if there are no modifications to the length
    ///
    /// Positive is down (+y), negative is up (-y)
    /// TODO: I don't think this is used
    fn get_signed_length(&self, stem_direction: &StemDirection) -> f32;

    /// Get the (unsigned) length of the stem if there are no modifications to the length
    fn get_length(&self, stem_direction: &StemDirection) -> f32 {
        self.get_signed_length(stem_direction).abs()
    }
}

impl HasStem for StemAnchors {
    fn get_stem_anchor(&self, stem_direction: &StemDirection) -> Vector {
        match stem_direction {
            StemDirection::UP => self.stem_up_se,
            StemDirection::DOWN => self.stem_down_nw,
        }
    }

    fn get_signed_length(&self, stem_direction: &StemDirection) -> f32 {
        let anchor = self.get_stem_anchor(stem_direction);
        let sign = match stem_direction {
            StemDirection::UP => -1.,
            StemDirection::DOWN => 1.,
        };
        sign * 3.5 * STANDARD_STAFF_SPACING - anchor.y
    }
}

impl HasStem for StemNoteMeta {
    fn get_stem_anchor(&self, stem_direction: &StemDirection) -> Vector {
        self.stem_anchors.get_stem_anchor(stem_direction)
    }

    fn get_signed_length(&self, stem_direction: &StemDirection) -> f32 {
        self.stem_anchors.get_signed_length(stem_direction)
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
        let advance_widths = &value.glyph_advance_widths;

        let notehead_black_stem_anchors =
            StemAnchors::try_from(&value.glyphs_with_anchors.notehead_black)?;

        Ok(FontMeta {
            font_iced,
            font_name: value.font_name,
            barlines_meta: BarlinesMeta {
                single_advance_width: advance_widths.barline_single * STANDARD_STAFF_SPACING,
                thin_thickness: value.engraving_defaults.thin_barline_thickness
                    * STANDARD_STAFF_SPACING,
            },
            engraving_defaults: EngravingDefaults {
                staff_line_thickness: value.engraving_defaults.staff_line_thickness
                    * STANDARD_STAFF_SPACING,
                stem_thickness: value.engraving_defaults.stem_thickness * STANDARD_STAFF_SPACING,
            },
            notes_meta: NotesMeta {
                whole_note: WholeNoteMeta {
                    advance_width: advance_widths.note_whole * STANDARD_STAFF_SPACING,
                },
                half_note: StemNoteMeta::from_smufl_units(
                    advance_widths.note_half_up,
                    advance_widths.note_half_down,
                    StemAnchors::try_from(&value.glyphs_with_anchors.notehead_half)?,
                ),
                quarter_note: StemNoteMeta::from_smufl_units(
                    advance_widths.note_half_up,
                    advance_widths.note_half_down,
                    notehead_black_stem_anchors.clone(),
                ),
                note_8th: StemNoteMeta::from_smufl_units(
                    advance_widths.note_8th_up,
                    advance_widths.note_8th_down,
                    notehead_black_stem_anchors.clone(),
                ),
                note_16th: StemNoteMeta::from_smufl_units(
                    advance_widths.note_16th_up,
                    advance_widths.note_16th_down,
                    notehead_black_stem_anchors.clone(),
                ),
                note_32nd: StemNoteMeta::from_smufl_units(
                    advance_widths.note_32nd_up,
                    advance_widths.note_32nd_down,
                    notehead_black_stem_anchors.clone(),
                ),
                note_64th: StemNoteMeta::from_smufl_units(
                    advance_widths.note_64th_up,
                    advance_widths.note_64th_down,
                    notehead_black_stem_anchors.clone(),
                ),
                note_128th: StemNoteMeta::from_smufl_units(
                    advance_widths.note_128th_up,
                    advance_widths.note_128th_down,
                    notehead_black_stem_anchors.clone(),
                ),
            },
            rests_meta: RestsMeta {
                rests: from_fn(|i| {
                    let advance_width = match i {
                        0 => advance_widths.rest_whole,
                        1 => advance_widths.rest_half,
                        2 => advance_widths.rest_quarter,
                        3 => advance_widths.rest_8th,
                        4 => advance_widths.rest_16th,
                        5 => advance_widths.rest_32nd,
                        6 => advance_widths.rest_64th,
                        7 => advance_widths.rest_128th,
                        _ => panic!("Rests shorter than 128th have not been implemented"),
                    } * STANDARD_STAFF_SPACING;

                    RestMeta { advance_width }
                }),
            },
            accidentals_meta: AccidentalsMeta {
                sharp_advance_width: advance_widths.accidental_sharp * STANDARD_STAFF_SPACING,
                natural_advance_width: advance_widths.accidental_natural * STANDARD_STAFF_SPACING,
                flat_advance_width: advance_widths.accidental_flat * STANDARD_STAFF_SPACING,
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
    note_64th_up: f32,
    note_64th_down: f32,
    note_128th_up: f32,
    note_128th_down: f32,
    rest_whole: f32,
    rest_half: f32,
    rest_quarter: f32,
    rest_8th: f32,
    rest_16th: f32,
    rest_32nd: f32,
    rest_64th: f32,
    rest_128th: f32,
    barline_single: f32,
    accidental_sharp: f32,
    accidental_natural: f32,
    accidental_flat: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGlyphsWithAnchors {
    notehead_whole: RawAnchors,
    notehead_half: RawAnchors,
    notehead_black: RawAnchors,
    flag_8th_up: RawAnchors,
    flag_8th_down: RawAnchors,
    flag_16th_up: RawAnchors,
    flag_16th_down: RawAnchors,
    flag_32nd_up: RawAnchors,
    flag_32nd_down: RawAnchors,
    flag_64th_up: RawAnchors,
    flag_64th_down: RawAnchors,
    flag_128th_up: RawAnchors,
    flag_128th_down: RawAnchors,
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

impl TryFrom<&RawAnchors> for StemAnchors {
    type Error = DeserializeError;

    fn try_from(value: &RawAnchors) -> Result<Self, Self::Error> {
        let stem_up_se = value
            .stem_up_se
            .map(parse_raw_vector)
            .ok_or(DeserializeError)?;
        let stem_down_nw = value
            .stem_down_nw
            .map(parse_raw_vector)
            .ok_or(DeserializeError)?;

        Ok(Self {
            stem_up_se,
            stem_down_nw,
        })
    }
}

pub trait GetAdvanceWidth {
    fn get_advance_width(&self, stem_direction: &StemDirection) -> f32;
}

impl GetAdvanceWidth for WholeNoteMeta {
    fn get_advance_width(&self, _stem_direction: &StemDirection) -> f32 {
        self.advance_width
    }
}

impl GetAdvanceWidth for StemNoteMeta {
    fn get_advance_width(&self, stem_direction: &StemDirection) -> f32 {
        match stem_direction {
            StemDirection::UP => self.stem_up_advance_width,
            StemDirection::DOWN => self.stem_down_advance_width,
        }
    }
}

/// Parse a vector from the raw metadata, making sure to apply the appropriate transformations
fn parse_raw_vector(raw: (f32, f32)) -> Vector {
    Vector::new(
        raw.0 * STANDARD_STAFF_SPACING,
        raw.1 * -STANDARD_STAFF_SPACING,
    )
}

// TODO: Better error handling
#[derive(Debug)]
struct DeserializeError;
impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deserialize Error")
    }
}
