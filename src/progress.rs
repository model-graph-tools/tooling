//! Terminal progress bars, step headers, and status reporting.

use console::{Emoji, style, truncate_str};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::Instant;

/// Column width for left-aligning progress bar names.
const NAME_WIDTH: usize = 30;

static COG: Emoji<'_, '_> = Emoji("\u{2699}\u{fe0f}  ", ""); // step 1: prepare
static MAG: Emoji<'_, '_> = Emoji("\u{1f50d}  ", ""); // step 2: analyze
static PACKAGE: Emoji<'_, '_> = Emoji("\u{1f4e6}  ", ""); // step 3: build
static BROOM: Emoji<'_, '_> = Emoji("\u{1f9f9}  ", ""); // step 4: cleanup
static SPARKLE: Emoji<'_, '_> = Emoji("\u{2728}  ", ":-)  "); // done

/// Prints a numbered step header with an emoji prefix (e.g. `[1/4] wrench Preparing...`).
pub fn step_header(step: u32, total: u32, description: &str) {
    let emoji = match step {
        1 => COG,
        2 => MAG,
        3 => PACKAGE,
        4 => BROOM,
        _ => COG,
    };
    println!(
        "{} {}{}",
        style(format!("[{}/{}]", step, total)).bold().dim(),
        emoji,
        description
    );
}

/// Prints elapsed time since `instant` as a "Done in ..." message.
pub fn done(instant: Instant) {
    println!(
        "\n{}Done in {}",
        SPARKLE,
        style(HumanDuration(instant.elapsed())).cyan()
    );
}

// ------------------------------------------------------ analysis status

/// Captures the outcome of a container operation for summary reporting.
#[derive(Clone)]
pub struct CommandStatus {
    pub identifier: String,
    pub success: bool,
    pub error_message: String,
}

impl CommandStatus {
    /// Creates a successful status.
    pub fn success(identifier: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: true,
            error_message: String::new(),
        }
    }

    /// Creates a failed status with an error message.
    pub fn error(identifier: &str, error_message: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: false,
            error_message: error_message.to_string(),
        }
    }

    /// Creates a status from a `Result`, mapping `Err` to an error status.
    pub fn from_result<T>(identifier: &str, result: &anyhow::Result<T>) -> Self {
        match result {
            Ok(_) => Self::success(identifier),
            Err(e) => Self::error(identifier, &e.to_string()),
        }
    }
}

/// Prints a summary of failed operations, if any.
pub fn summary(count: usize, status: &[CommandStatus]) {
    let failed: Vec<_> = status.iter().filter(|s| !s.success).collect();
    if !failed.is_empty() {
        println!();
        for s in &failed {
            println!(
                "  {} {} {}",
                style("\u{2717}").red().bold(),
                style(&s.identifier).cyan(),
                style(&s.error_message).red()
            );
        }
        println!(
            "\n  {} of {} resource(s) failed",
            style(failed.len()).red().bold(),
            style(count).cyan()
        );
    }
}

// ------------------------------------------------------ progress

/// A named spinner progress bar for tracking long-running operations.
#[derive(Clone)]
pub struct Progress {
    name: String,
    bar: ProgressBar,
}

impl Progress {
    /// Creates a progress bar and adds it to a `MultiProgress` group.
    pub fn join(multi_progress: &MultiProgress, name: &str) -> Progress {
        let progress = Progress {
            name: name.to_string(),
            bar: Self::spinner(),
        };
        progress.bar.enable_steady_tick(Duration::from_millis(100));
        multi_progress.add(progress.bar.clone());
        progress.bar.set_message(style(name).cyan().to_string());
        progress
    }

    /// Creates a hidden progress bar that produces no terminal output.
    pub fn hidden(name: &str) -> Progress {
        Progress {
            name: name.to_string(),
            bar: ProgressBar::hidden(),
        }
    }

    /// Creates a standalone progress bar.
    pub fn new(name: &str) -> Progress {
        let progress = Progress {
            name: name.to_string(),
            bar: Self::spinner(),
        };
        progress.bar.enable_steady_tick(Duration::from_millis(100));
        progress.bar.set_message(style(name).cyan().to_string());
        progress
    }

    /// Creates a braille-style spinner progress bar.
    fn spinner() -> ProgressBar {
        ProgressBar::new_spinner().with_style(
            ProgressStyle::default_spinner()
                .tick_strings(&[
                    "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}",
                    "\u{2826}", "\u{2827}", "\u{2807}", "\u{280f}", " ",
                ])
                .template("  {spinner:.dim.bold} {wide_msg}")
                .expect("Invalid spinner template"),
        )
    }

    /// Returns the style used when a spinner has finished (no tick animation).
    fn finished_style() -> ProgressStyle {
        ProgressStyle::default_spinner()
            .template("  {wide_msg}")
            .expect("Invalid template")
    }

    /// Updates the spinner message with the current operation status.
    pub fn show_progress(&self, text: &str) {
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        self.bar.set_message(format!(
            "{} {}",
            style(padded).cyan(),
            style(truncate_str(text, 80, "...")).dim()
        ));
    }

    /// Finishes the spinner with a green checkmark and optional status text.
    pub fn finish_success(&self, status: Option<&str>) {
        self.bar.set_style(Self::finished_style());
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        let msg = match status {
            Some(s) => format!(
                "{} {} {}",
                style("\u{2713}").green().bold(),
                style(padded).cyan(),
                style(s).dim()
            ),
            None => format!(
                "{} {}",
                style("\u{2713}").green().bold(),
                style(padded).cyan()
            ),
        };
        self.bar.finish_with_message(msg);
    }

    /// Abandons the spinner with a red cross and error message.
    pub fn finish_error(&self, err: &str) {
        self.bar.set_style(Self::finished_style());
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        self.bar.abandon_with_message(format!(
            "{} {} {}",
            style("\u{2717}").red().bold(),
            style(padded).cyan(),
            style(err).red()
        ));
    }
}
