#[derive(Debug)]
pub struct Fraction {
    numerator: u8,
    denominator: u8,
}

impl From<u8> for Fraction {
    fn from(value: u8) -> Self {
        Fraction {
            numerator: value,
            denominator: 1,
        }
    }
}
