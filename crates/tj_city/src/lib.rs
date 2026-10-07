//! City-level data models.

/// A normalized development density in the inclusive range `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density(f32);

impl Density {
    /// Creates a density, returning `None` when the value is non-finite or outside `0..=1`.
    pub fn new(value: f32) -> Option<Self> {
        (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(Self(value))
    }

    /// Returns the normalized density value.
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for Density {
    fn default() -> Self {
        Self(0.0)
    }
}

/// The two street-facing sides of a residential zone, each with its own density.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResidentialZone {
    pub left_density: Density,
    pub right_density: Density,
}

impl ResidentialZone {
    /// Creates a residential zone with independently specified side densities.
    pub fn new(left_density: Density, right_density: Density) -> Self {
        Self {
            left_density,
            right_density,
        }
    }
}

/// Basic city metadata.
pub struct City {
    pub name: String,
}

#[cfg(test)]
mod density_tests {
    use super::{Density, ResidentialZone};

    #[test]
    fn density_accepts_values_inclusive_of_zero_and_one() {
        assert_eq!(Density::new(0.0).unwrap().value(), 0.0);
        assert_eq!(Density::new(1.0).unwrap().value(), 1.0);
        assert_eq!(Density::new(0.42).unwrap().value(), 0.42);
    }

    #[test]
    fn density_rejects_out_of_range_and_non_finite_values() {
        for value in [-0.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(Density::new(value).is_none(), "accepted {value}");
        }
    }

    #[test]
    fn residential_zone_stores_independent_density_for_each_side() {
        let zone = ResidentialZone::new(Density::new(0.25).unwrap(), Density::new(0.75).unwrap());

        assert_eq!(zone.left_density.value(), 0.25);
        assert_eq!(zone.right_density.value(), 0.75);
    }
}
