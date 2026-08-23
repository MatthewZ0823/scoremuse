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
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
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

    // Y offset is in staff coordinates/units
    pub fn to_y_offset(&self) -> f32 {
        let p: f32 = self.to_staff_index().0 as f32;
        let b: f32 = Pitch::new(PitchClass::F, 5, None).to_staff_index().0 as f32;
        STANDARD_STAFF_SPACING / 2. * (b - p)
    }

    // Y offset is in staff coordinates/units
    // Returns None if the pitch would be lower than the lowest possible
    pub fn from_y_offset(y: f32) -> Option<Self> {
        let b: f32 = Pitch::new(PitchClass::F, 5, None).to_staff_index().0 as f32;
        let p = b - 2. * y / STANDARD_STAFF_SPACING;

        if p <= 0. {
            None
        } else {
            Some(Pitch::from(p as u8))
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

        12 + 12 * (self.octave as i32) + within_octave
    }

    /// Convert a pitch to a staff index
    fn to_staff_index(&self) -> StaffIndex {
        StaffIndex(self.octave * 7 + self.pitch_class as u8)
    }

    /// Convert a staff index to a pitch, with no accidentals
    fn from_staff_index(index: StaffIndex) -> Self {
        Self {
            pitch_class: FromPrimitive::from_u8(index.0 % 7).unwrap(),
            octave: index.0 / 7,
            accidental: None,
        }
    }
}

/// Staff index is the note's position in the staff, ignoring accidentals
/// i.e. C0, Cb0 and C#0 all map to 0, D0 maps to 1, E0 maps to 2, etc.
struct StaffIndex(u8);

// impl Ord for Pitch {
//     /// Lower pitched notes are "less than" higher pitched ones
//     /// TODO: Fix this
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.octave
//             .cmp(&other.octave)
//             .then(self.pitch_class.cmp(&other.pitch_class))
//     }
// }
//
// impl PartialOrd for Pitch {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
