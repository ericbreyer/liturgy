//! CSV utilities for liturgical calendars
//!
//! This module provides convenient functions for working with CSV data
//! from liturgical calendars, including parsing, filtering, and analysis.

use crate::calender::YearCalendarHandle;
use std::path::Path;
use anyhow::Result;

/// CSV record representing a single liturgical day
#[derive(Debug, Clone)]
pub struct LiturgicalDayRecord {
    pub date: String,
    pub day_name: String,
    pub season: String,
    pub color: String,
    pub rank: String,
    pub feast_name: Option<String>,
    pub commemorations: Vec<String>,
}

/// CSV utilities for liturgical calendar data
pub struct CsvUtils;

impl CsvUtils {
    /// Load liturgical year data from a CSV file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::csv_utils::CsvUtils;
    ///
    /// let records = CsvUtils::load_from_file("calendar_2025.csv")?;
    /// println!("Loaded {} days", records.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(_path: P) -> Result<Vec<LiturgicalDayRecord>> {
        // TODO: Implement CSV parsing logic
        todo!("Implement CSV file loading")
    }

    /// Generate a CSV file from a year calendar
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::{GenericCalendarHandle, csv_utils::CsvUtils};
    ///
    /// let calendar = GenericCalendarHandle::load_from_file("calendar_data/of.toml")?;
    /// let year_2025 = calendar.create_year_calendar(2025);
    /// CsvUtils::export_year_calendar(&year_2025, "output_2025.csv")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn export_year_calendar<P: AsRef<Path>>(
        calendar: &YearCalendarHandle,
        path: P,
    ) -> Result<()> {
        // Use the existing export functionality from YearCalendarHandle
        calendar.export_csv(path)?;
        Ok(())
    }

    /// Filter liturgical days by feast rank
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::csv_utils::CsvUtils;
    ///
    /// let records = CsvUtils::load_from_file("calendar_2025.csv")?;
    /// let major_feasts = CsvUtils::filter_by_rank(&records, "I");
    /// println!("Found {} major feasts", major_feasts.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn filter_by_rank<'a>(records: &'a [LiturgicalDayRecord], rank: &str) -> Vec<&'a LiturgicalDayRecord> {
        records.iter().filter(|record| record.rank == rank).collect()
    }

    /// Filter liturgical days by liturgical season
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::csv_utils::CsvUtils;
    ///
    /// let records = CsvUtils::load_from_file("calendar_2025.csv")?;
    /// let advent_days = CsvUtils::filter_by_season(&records, "Advent");
    /// println!("Found {} days in Advent", advent_days.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn filter_by_season<'a>(records: &'a [LiturgicalDayRecord], season: &str) -> Vec<&'a LiturgicalDayRecord> {
        records.iter().filter(|record| record.season == season).collect()
    }

    /// Get all unique feast names from records
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::csv_utils::CsvUtils;
    ///
    /// let records = CsvUtils::load_from_file("calendar_2025.csv")?;
    /// let feast_names = CsvUtils::extract_feast_names(&records);
    /// println!("Found {} unique feasts", feast_names.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn extract_feast_names(records: &[LiturgicalDayRecord]) -> Vec<String> {
        let mut feast_names: Vec<String> = records
            .iter()
            .filter_map(|record| record.feast_name.as_ref())
            .cloned()
            .collect();
        
        feast_names.sort();
        feast_names.dedup();
        feast_names
    }

    /// Generate summary statistics from liturgical calendar data
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use liturgy::csv_utils::CsvUtils;
    ///
    /// let records = CsvUtils::load_from_file("calendar_2025.csv")?;
    /// let stats = CsvUtils::generate_statistics(&records);
    /// println!("Calendar statistics: {:#?}", stats);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn generate_statistics(records: &[LiturgicalDayRecord]) -> CalendarStatistics {
        let total_days = records.len();
        let feast_days = records.iter().filter(|r| r.feast_name.is_some()).count();
        let ferial_days = total_days - feast_days;
        
        let mut season_counts = std::collections::HashMap::new();
        let mut color_counts = std::collections::HashMap::new();
        let mut rank_counts = std::collections::HashMap::new();
        
        for record in records {
            *season_counts.entry(record.season.clone()).or_insert(0) += 1;
            *color_counts.entry(record.color.clone()).or_insert(0) += 1;
            *rank_counts.entry(record.rank.clone()).or_insert(0) += 1;
        }
        
        CalendarStatistics {
            total_days,
            feast_days,
            ferial_days,
            season_counts,
            color_counts,
            rank_counts,
        }
    }
}

/// Statistics summary for a liturgical calendar
#[derive(Debug, Clone)]
pub struct CalendarStatistics {
    pub total_days: usize,
    pub feast_days: usize,
    pub ferial_days: usize,
    pub season_counts: std::collections::HashMap<String, usize>,
    pub color_counts: std::collections::HashMap<String, usize>,
    pub rank_counts: std::collections::HashMap<String, usize>,
}

/// Convenience functions for working with CSV data
impl LiturgicalDayRecord {
    /// Check if this day is a major feast (rank I)
    pub fn is_major_feast(&self) -> bool {
        self.rank == "I"
    }

    /// Check if this day is a feast day
    pub fn is_feast(&self) -> bool {
        self.feast_name.is_some()
    }

    /// Check if this day is in a specific season
    pub fn is_in_season(&self, season: &str) -> bool {
        self.season == season
    }

    /// Get the primary liturgical color
    pub fn primary_color(&self) -> &str {
        // Handle cases where multiple colors might be specified
        self.color.split('/').next().unwrap_or(&self.color)
    }
}
