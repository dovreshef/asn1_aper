pub const LENGTH_DET_SHORT: u8 = 0b0000_0000;
pub const LENGTH_DET_LONG: u8 = 0b1000_0000;
pub const LENGTH_DET_FRAG: u8 = 0b1100_0000;

pub const LENGTH_MASK_SHORT: u8 = 0b0111_1111;
pub const LENGTH_MASK_LONG: u8 = 0b0011_1111;

/// An interval that describes the limits on some value.
/// To indicate something is unbounded, set `min` and `max` to `None`.
#[derive(Debug, Copy, Clone)]
pub struct Constraint {
    min: Option<i64>,
    max: Option<i64>,
}

impl Constraint {
    /// Construct a new `Constraint`.
    pub fn new(min: Option<i64>, max: Option<i64>) -> Constraint {
        Constraint { min, max }
    }

    /// Get the lower bound.
    pub fn min(&self) -> Option<i64> {
        self.min
    }

    /// Get the upper bound.
    pub fn max(&self) -> Option<i64> {
        self.max
    }
}

/// A pair of `Constraint`s that describes the constraints on the value (if applicable) and encoded size of a type.
/// A value is considered unconstrained if `value` and `size` are both set to `None`.
#[derive(Debug, Copy, Clone)]
pub struct Constraints {
    pub value: Option<Constraint>,
    pub size: Option<Constraint>,
}

pub const UNCONSTRAINED: Constraints = Constraints {
    value: None,
    size: None,
};
