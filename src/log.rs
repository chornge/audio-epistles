use chrono::{Datelike, Local};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

// Write a log entry to the correct log file for the current month
fn write_log_entry(log_message: &str) -> std::io::Result<()> {
    let current_date = Local::now();
    let current_month = get_current_month_folder();

    // Format the log file name (e.g., 2024-02-11.log)
    let log_file_name = format!(
        "logs/{}/{}.log",
        current_month,
        current_date.format("%Y-%m-%d")
    );

    // Open log file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file_name)?;

    // Write the log message
    writeln!(
        file,
        "[{}] - {}",
        current_date.format("%Y-%m-%d %H:%M:%S"),
        log_message
    )?;

    Ok(())
}

pub fn log_event(event_message: &str) -> std::io::Result<()> {
    clear_logs_for_new_month()?; // if applicable

    write_log_entry(event_message)?;

    Ok(())
}

fn get_month_folder(month: u32) -> String {
    match month {
        1 => "1-jan",
        2 => "2-feb",
        3 => "3-mar",
        4 => "4-apr",
        5 => "5-may",
        6 => "6-jun",
        7 => "7-jul",
        8 => "8-aug",
        9 => "9-sep",
        10 => "10-oct",
        11 => "11-nov",
        12 => "12-dec",
        _ => unreachable!(),
    }
    .to_string()
}

// Get the folder name based on the current month
fn get_current_month_folder() -> String {
    let current_date = Local::now();
    get_month_folder(current_date.month())
}

fn get_next_month_folder() -> String {
    let current_date = Local::now();
    let next_month = if current_date.month() == 12 {
        1 // Wrap around to jan
    } else {
        current_date.month() + 1
    };
    get_month_folder(next_month)
}

fn clear_logs_for_new_month() -> std::io::Result<()> {
    let current_date = Local::now();
    let current_day = current_date.day();

    // Only clear logs if current date is the 22nd or later
    if current_day >= 22 {
        let next_month = get_next_month_folder();
        let next_month_dir = format!("logs/{}", next_month);

        if Path::new(&next_month_dir).exists() {
            for entry in fs::read_dir(&next_month_dir)? {
                let entry = entry?;
                fs::remove_file(entry.path())?;
            }
            // println!("Cleared logs for: {}", next_month);
        }
    }

    Ok(())
}
