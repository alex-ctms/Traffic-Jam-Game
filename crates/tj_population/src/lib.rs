//! Aggregate population and employment counts.

/// Current citywide resident and job counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopulationStats {
    pub population: u64,
    pub jobs: u64,
}

impl PopulationStats {
    /// Format the counts for the `/pop_stats` developer console command.
    pub fn format_console_output(self) -> String {
        format!("Population: {}\nJobs: {}", self.population, self.jobs)
    }
}

pub struct Household {
    pub id: u64,
    pub size: u32,
}

#[cfg(test)]
mod tests {
    use super::PopulationStats;

    #[test]
    fn console_output_reports_population_and_jobs() {
        assert_eq!(
            PopulationStats {
                population: 1250,
                jobs: 840,
            }
            .format_console_output(),
            "Population: 1250\nJobs: 840"
        );
    }
}
