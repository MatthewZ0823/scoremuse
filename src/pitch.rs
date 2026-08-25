use num_traits::FromPrimitive;

use crate::constants::STANDARD_STAFF_SPACING;

#[allow(dead_code)]
#[derive(FromPrimitive, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum PitchClass {
    C = 0,
    D = 1,
    E = 2,
    F = 3,
    G = 4,
    A = 5,
    B = 6,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

impl Accidental {
    pub fn to_glyph(&self) -> &'static str {
        match self {
            Self::Sharp => "\u{E262}",
            Self::Natural => "\u{E261}",
            Self::Flat => "\u{E260}",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pitch {
    pub pitch_class: PitchClass,
    pub octave: u8,
    pub accidental: Option<Accidental>,
}

impl Pitch {
    pub const fn new(pitch_class: PitchClass, octave: u8, accidental: Option<Accidental>) -> Self {
        Pitch {
            pitch_class,
            octave,
            accidental,
        }
    }

    // Y is expected to be in staff coordinates/units
    pub fn to_y(&self) -> f32 {
        let p: f32 = self.to_staff_index().0 as f32;
        let b: f32 = Pitch::new(PitchClass::F, 5, None).to_staff_index().0 as f32;
        STANDARD_STAFF_SPACING / 2. * (b - p)
    }

    // Y is expected to be in staff coordinates/units
    // Returns None if the pitch would be lower than the lowest possible
    pub fn from_y(y: f32) -> Option<Self> {
        let b: f32 = Pitch::new(PitchClass::F, 5, None).to_staff_index().0 as f32;
        let p = b - 2. * y / STANDARD_STAFF_SPACING;

        if p <= 0. {
            None
        } else {
            Some(Pitch::from_staff_index(StaffIndex(p.round() as u8)))
        }
    }

    pub fn to_midi_note_number(&self) -> i32 {
        let within_octave = match self.pitch_class {
            PitchClass::C => 0,
            PitchClass::D => 2,
            PitchClass::E => 4,
            PitchClass::F => 5,
            PitchClass::G => 7,
            PitchClass::A => 9,
            PitchClass::B => 11,
        };

        // TODO: Change the default case of accidental to go with the key signature
        let accidental = self.accidental.map_or(0, |a| match a {
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
        });

        12 + 12 * (self.octave as i32) + within_octave + accidental
    }

    /// Convert a pitch to a staff index
    pub fn to_staff_index(&self) -> StaffIndex {
        StaffIndex(self.octave * 7 + self.pitch_class as u8)
    }

    /// Convert a staff index to a pitch, with no accidentals
    pub fn from_staff_index(index: StaffIndex) -> Self {
        Self {
            pitch_class: FromPrimitive::from_u8(index.0 % 7).unwrap(),
            octave: index.0 / 7,
            accidental: None,
        }
    }

    /// Toggle the accidental
    /// eg. If the note is flat and `accidental` is flat, then the note's accidental becomes None
    /// eg. If the note is flat and `accidental` is sharp, then the note's accidental becomes sharp
    pub fn toggle_accidental(&self, accidental: Accidental) -> Self {
        let new_accidental = match self.accidental {
            None => Some(accidental),
            Some(a) => {
                if accidental == a {
                    None
                } else {
                    Some(accidental)
                }
            }
        };

        Self {
            pitch_class: self.pitch_class,
            octave: self.octave,
            accidental: new_accidental,
        }
    }
}

/// Staff index is the note's position in the staff, ignoring accidentals
/// i.e. C0, Cb0 and C#0 all map to 0, D0 maps to 1, E0 maps to 2, etc.
pub struct StaffIndex(pub u8);
