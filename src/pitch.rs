use std::cmp::Ordering;

#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
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
        let p: u8 = (*self).into();
        let b: u8 = Pitch::new(PitchClass::E, 4).into();
        0.125 * (p - b) as f32
    }
}

impl From<Pitch> for u8 {
    /// C0 maps to 0 and each pitch higher increases
    fn from(value: Pitch) -> Self {
        value.octave * 7 + value.pitch_class as u8
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
