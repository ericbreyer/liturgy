use calendar_calc::GenericCalendarHandle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing feast merging functionality...");

    // Load the base OF calendar
    let base_calendar = GenericCalendarHandle::load_from_file("calendar_data/of.toml")?;
    println!("Loaded base calendar: {}", base_calendar.name());

    // Load with US extensions using the new functionality
    let extended_calendar = GenericCalendarHandle::load_with_extensions(
        "calendar_data/of.toml",
        &["calendar_data/of-us-extensions.toml"],
    )?;
    println!(
        "Loaded calendar with US extensions: {}",
        extended_calendar.name()
    );

    // Create year calendars
    let base_year = base_calendar.create_year_calendar(2025);
    let extended_year = extended_calendar.create_year_calendar(2025);

    println!(
        "Base calendar generated successfully for {}",
        base_year.year()
    );
    println!(
        "Extended calendar generated successfully for {}",
        extended_year.year()
    );

    // Export both to see the difference
    base_year.export_csv("calendar_of_2025_base.csv")?;
    extended_year.export_csv("calendar_of_2025_with_us_extensions.csv")?;

    println!("Exported both calendars for comparison");
    println!("- calendar_of_2025_base.csv (base calendar)");
    println!("- calendar_of_2025_with_us_extensions.csv (with US extensions)");

    Ok(())
}
