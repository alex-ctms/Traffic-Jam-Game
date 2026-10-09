//! Travel demand generation: turns households into daily commute trips.

use tj_core::EntityId;

pub struct TripDemand {
    pub origin: u64,
    pub destination: u64,
}

/// Identifier of a zone (residential origin or job destination).
pub type ZoneId = u64;

/// A household located in a residential zone, ready to produce a daily trip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HouseholdOrigin {
    pub household_id: EntityId,
    pub home_zone: ZoneId,
}

/// One daily commute from a household's home zone to a job zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trip {
    pub household_id: EntityId,
    pub origin_zone: ZoneId,
    pub destination_zone: ZoneId,
}

/// Errors raised when the probability matrix or household input is invalid.
#[derive(Clone, Debug, PartialEq)]
pub enum TripGenError {
    /// The matrix has no job zones to choose from.
    NoJobZones,
    /// A row does not have one weight per job zone.
    RowLengthMismatch {
        home_zone: ZoneId,
        expected: usize,
        actual: usize,
    },
    /// A weight is negative, NaN or infinite.
    InvalidWeight {
        home_zone: ZoneId,
        job_zone: ZoneId,
        weight: f64,
    },
    /// A row's weights sum to zero, so no job zone can be chosen.
    EmptyRow { home_zone: ZoneId },
    /// A row's weights overflow when summed and cannot be sampled reliably.
    InvalidRowTotal { home_zone: ZoneId },
    /// The same home zone appears in more than one row.
    DuplicateHomeZone { home_zone: ZoneId },
    /// A job zone appears in more than one column.
    DuplicateJobZone { job_zone: ZoneId },
    /// A household lives in a zone that has no row in the matrix.
    UnknownHomeZone {
        household_id: EntityId,
        home_zone: ZoneId,
    },
}

/// Home-zone → job-zone probability matrix.
///
/// Each row holds non-negative weights, one per job zone. Weights are
/// normalised per row, so they do not need to sum to 1.
#[derive(Clone, Debug, PartialEq)]
pub struct TripProbabilityMatrix {
    job_zones: Vec<ZoneId>,
    rows: Vec<(ZoneId, Vec<f64>)>,
}

impl TripProbabilityMatrix {
    /// Build and validate a matrix from `(home_zone, weights)` rows.
    pub fn new(
        job_zones: Vec<ZoneId>,
        rows: Vec<(ZoneId, Vec<f64>)>,
    ) -> Result<Self, TripGenError> {
        if job_zones.is_empty() {
            return Err(TripGenError::NoJobZones);
        }
        for (index, &job_zone) in job_zones.iter().enumerate() {
            if job_zones[..index].contains(&job_zone) {
                return Err(TripGenError::DuplicateJobZone { job_zone });
            }
        }
        for (index, (home_zone, weights)) in rows.iter().enumerate() {
            let home_zone = *home_zone;
            if rows[..index].iter().any(|(zone, _)| *zone == home_zone) {
                return Err(TripGenError::DuplicateHomeZone { home_zone });
            }
            if weights.len() != job_zones.len() {
                return Err(TripGenError::RowLengthMismatch {
                    home_zone,
                    expected: job_zones.len(),
                    actual: weights.len(),
                });
            }
            for (&job_zone, &weight) in job_zones.iter().zip(weights) {
                if !weight.is_finite() || weight < 0.0 {
                    return Err(TripGenError::InvalidWeight {
                        home_zone,
                        job_zone,
                        weight,
                    });
                }
            }
            let total = weights.iter().sum::<f64>();
            if !total.is_finite() {
                return Err(TripGenError::InvalidRowTotal { home_zone });
            }
            if total <= 0.0 {
                return Err(TripGenError::EmptyRow { home_zone });
            }
        }
        Ok(Self { job_zones, rows })
    }

    /// Job zones in column order.
    pub fn job_zones(&self) -> &[ZoneId] {
        &self.job_zones
    }

    /// Normalised probability of a trip from `home_zone` ending in `job_zone`.
    pub fn probability(&self, home_zone: ZoneId, job_zone: ZoneId) -> Option<f64> {
        let weights = self.row(home_zone)?;
        let column = self.job_zones.iter().position(|&zone| zone == job_zone)?;
        Some(weights[column] / weights.iter().sum::<f64>())
    }

    fn row(&self, home_zone: ZoneId) -> Option<&[f64]> {
        self.rows
            .iter()
            .find(|(zone, _)| *zone == home_zone)
            .map(|(_, weights)| weights.as_slice())
    }

    /// Pick a job zone for `home_zone` using a uniform sample `u` in `[0, 1)`.
    fn sample(&self, home_zone: ZoneId, u: f64) -> Option<ZoneId> {
        let weights = self.row(home_zone)?;
        let target = u * weights.iter().sum::<f64>();
        let mut cumulative = 0.0;
        let mut last_positive = None;
        for (&job_zone, &weight) in self.job_zones.iter().zip(weights) {
            if weight <= 0.0 {
                continue;
            }
            cumulative += weight;
            last_positive = Some(job_zone);
            if target < cumulative {
                return Some(job_zone);
            }
        }
        // Floating-point rounding can leave `target` at the very top of the range.
        last_positive
    }
}

/// Deterministic SplitMix64 generator so a seed always yields the same day.
#[derive(Clone, Debug)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform sample in `[0, 1)` with 53 bits of precision.
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Generate one daily [`Trip`] per household, choosing each job zone from
/// the household's home-zone row of `matrix`. The same `seed` and input
/// always produce the same trips, in household order.
pub fn generate_daily_trips(
    households: &[HouseholdOrigin],
    matrix: &TripProbabilityMatrix,
    seed: u64,
) -> Result<Vec<Trip>, TripGenError> {
    let mut rng = SplitMix64(seed);
    households
        .iter()
        .map(|household| {
            let destination_zone = matrix.sample(household.home_zone, rng.next_f64()).ok_or(
                TripGenError::UnknownHomeZone {
                    household_id: household.household_id,
                    home_zone: household.home_zone,
                },
            )?;
            Ok(Trip {
                household_id: household.household_id,
                origin_zone: household.home_zone,
                destination_zone,
            })
        })
        .collect()
}

#[cfg(test)]
mod daily_trip_tests {
    use super::*;

    fn households(count: u64, home_zone: ZoneId) -> Vec<HouseholdOrigin> {
        (0..count)
            .map(|household_id| HouseholdOrigin {
                household_id,
                home_zone,
            })
            .collect()
    }

    fn two_home_matrix() -> TripProbabilityMatrix {
        TripProbabilityMatrix::new(
            vec![100, 200, 300],
            vec![(1, vec![0.7, 0.2, 0.1]), (2, vec![0.0, 1.0, 3.0])],
        )
        .unwrap()
    }

    #[test]
    fn every_household_gets_exactly_one_trip_from_its_home_to_a_job_zone() {
        let matrix = two_home_matrix();
        let mut input = households(50, 1);
        input.extend((50..120).map(|household_id| HouseholdOrigin {
            household_id,
            home_zone: 2,
        }));

        let trips = generate_daily_trips(&input, &matrix, 42).unwrap();

        assert_eq!(trips.len(), input.len());
        for (household, trip) in input.iter().zip(&trips) {
            assert_eq!(trip.household_id, household.household_id);
            assert_eq!(trip.origin_zone, household.home_zone);
            assert!(matrix.job_zones().contains(&trip.destination_zone));
            let (from, to) = (trip.origin_zone, trip.destination_zone);
            assert!(matrix.probability(from, to).unwrap() > 0.0);
        }
    }

    #[test]
    fn destinations_follow_the_probability_matrix() {
        let matrix = two_home_matrix();
        let input = households(20_000, 1);
        let trips = generate_daily_trips(&input, &matrix, 7).unwrap();

        for (job_zone, expected) in [(100, 0.7), (200, 0.2), (300, 0.1)] {
            let count = trips
                .iter()
                .filter(|trip| trip.destination_zone == job_zone)
                .count();
            let share = count as f64 / trips.len() as f64;
            assert!(
                (share - expected).abs() < 0.02,
                "job zone {job_zone}: share {share} vs expected {expected}"
            );
        }
    }

    #[test]
    fn zero_weight_job_zones_are_never_chosen() {
        let matrix = two_home_matrix();
        let input = households(5_000, 2);
        let trips = generate_daily_trips(&input, &matrix, 3).unwrap();
        assert!(trips.iter().all(|trip| trip.destination_zone != 100));
    }

    #[test]
    fn same_seed_is_deterministic_and_different_seeds_vary() {
        let matrix = two_home_matrix();
        let input = households(500, 1);
        let first = generate_daily_trips(&input, &matrix, 99).unwrap();
        assert_eq!(first, generate_daily_trips(&input, &matrix, 99).unwrap());
        assert_ne!(first, generate_daily_trips(&input, &matrix, 100).unwrap());
    }

    #[test]
    fn no_households_produce_no_trips() {
        assert_eq!(generate_daily_trips(&[], &two_home_matrix(), 1), Ok(vec![]));
    }

    #[test]
    fn household_in_unknown_zone_is_rejected() {
        let input = [HouseholdOrigin {
            household_id: 9,
            home_zone: 77,
        }];
        assert_eq!(
            generate_daily_trips(&input, &two_home_matrix(), 1),
            Err(TripGenError::UnknownHomeZone {
                household_id: 9,
                home_zone: 77,
            })
        );
    }

    #[test]
    fn invalid_matrices_are_rejected() {
        assert_eq!(
            TripProbabilityMatrix::new(vec![], vec![]),
            Err(TripGenError::NoJobZones)
        );
        assert_eq!(
            TripProbabilityMatrix::new(vec![1, 2], vec![(5, vec![1.0])]),
            Err(TripGenError::RowLengthMismatch {
                home_zone: 5,
                expected: 2,
                actual: 1,
            })
        );
        assert_eq!(
            TripProbabilityMatrix::new(vec![1], vec![(5, vec![-1.0])]),
            Err(TripGenError::InvalidWeight {
                home_zone: 5,
                job_zone: 1,
                weight: -1.0,
            })
        );
        assert!(matches!(
            TripProbabilityMatrix::new(vec![1], vec![(5, vec![f64::NAN])]),
            Err(TripGenError::InvalidWeight { .. })
        ));
        assert_eq!(
            TripProbabilityMatrix::new(vec![1, 2], vec![(5, vec![0.0, 0.0])]),
            Err(TripGenError::EmptyRow { home_zone: 5 })
        );
        assert_eq!(
            TripProbabilityMatrix::new(vec![1], vec![(5, vec![1.0]), (5, vec![2.0])]),
            Err(TripGenError::DuplicateHomeZone { home_zone: 5 })
        );
        assert_eq!(
            TripProbabilityMatrix::new(vec![1, 1], vec![(5, vec![1.0, 2.0])]),
            Err(TripGenError::DuplicateJobZone { job_zone: 1 })
        );
        assert_eq!(
            TripProbabilityMatrix::new(vec![1, 2], vec![(5, vec![f64::MAX, f64::MAX])]),
            Err(TripGenError::InvalidRowTotal { home_zone: 5 })
        );
    }

    #[test]
    fn probabilities_are_normalised_per_row() {
        let matrix = two_home_matrix();
        assert_eq!(matrix.probability(2, 300), Some(0.75));
        assert_eq!(matrix.probability(2, 999), None);
        assert_eq!(matrix.probability(999, 300), None);
    }
}
