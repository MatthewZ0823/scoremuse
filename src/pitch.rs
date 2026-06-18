use std::cmp::Ordering;

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
pub struct Pitch {
    pub pitch_class: PitchClass,
    pub octave: u8,
}

impl Pitch {
    pub const fn new(pitch_class: PitchClass, octave: u8) -> Self {
        Pitch {
            pitch_class,
            octave,
        }
    }

    // Y offset is in staff coordinates/units
    pub fn to_y_offset(&self) -> f32 {
        let p: f32 = u8::from(*self) as f32;
        let b: f32 = u8::from(Pitch::new(PitchClass::F, 5)) as f32;
        STANDARD_STAFF_SPACING / 2. * (b - p) as f32
    }

    // Y offset is in staff coordinates/units
    // Returns None if the pitch would be lower than the lowest possible
    pub fn from_y_offset(y: f32) -> Option<Self> {
        let b: f32 = u8::from(Pitch::new(PitchClass::F, 5)) as f32;
        let p = b - 2. * y / STANDARD_STAFF_SPACING;

        if p <= 0. {
            None
        } else {
            Some(Pitch::from(p as u8))
        }
    }
}

impl From<Pitch> for u8 {
    /// C0 maps to 0 and each pitch higher increases
    fn from(value: Pitch) -> Self {
        value.octave * 7 + value.pitch_class as u8
    }
}

impl From<u8> for Pitch {
    /// C0 maps to 0 and each pitch higher increases
    fn from(value: u8) -> Self {
        Self {
            pitch_class: FromPrimitive::from_u8(value % 7).unwrap(),
            octave: value / 7,
        }
    }
}

impl Ord for Pitch {
    /// Lower pitched notes are "less than" higher pitched ones
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.octave
            .cmp(&other.octave)
            .then(self.pitch_class.cmp(&other.pitch_class))
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
