use crate::deadlock::DeadlockReport;
use crate::race::RaceReport;
use colored::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static ISSUE_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn report_deadlock(report: &DeadlockReport) {
    ISSUE_COUNT.fetch_add(1, Ordering::Relaxed);

    eprintln!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════╗".bright_red()
    );
    eprintln!(
        "{} {} {}",
        "║".bright_red(),
        "⚠️  DEADLOCK DETECTED".bright_red().bold(),
        "║".bright_red()
    );
    eprintln!(
        "{}\n",
        "╚══════════════════════════════════════════════════════════╝".bright_red()
    );

    eprintln!(
        "{}",
        format!("Cycle Length: {} locks", report.cycle_length).yellow()
    );
    eprintln!("{}", "Lock Chain:".cyan().bold());

    for (i, entry) in report.lock_chain.iter().enumerate() {
        eprintln!(
            "  {} Lock #{} (Type: {:?}) held by Thread {}",
            format!("[{}]", i + 1).green(),
            entry.lock_id,
            entry.lock_type,
            entry.thread_id
        );

        eprintln!("    {}", "Backtrace:".magenta());
        for (j, frame) in entry.backtrace.iter().take(5).enumerate() {
            eprintln!(
                "      {} {}",
                format!("{}.", j + 1).dimmed(),
                frame.dimmed()
            );
        }
    }
    eprintln!();
}

pub fn report_race(report: &RaceReport) {
    ISSUE_COUNT.fetch_add(1, Ordering::Relaxed);

    eprintln!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════╗".bright_yellow()
    );
    eprintln!(
        "{} {} {}",
        "║".bright_yellow(),
        "⚡ RACE CONDITION DETECTED".bright_yellow().bold(),
        "║".bright_yellow()
    );
    eprintln!(
        "{}\n",
        "╚══════════════════════════════════════════════════════════╝".bright_yellow()
    );

    eprintln!(
        "{}",
        format!("Memory Address: 0x{:016x}", report.address).yellow()
    );

    eprintln!(
        "\n{} (Thread {})",
        "Access 1:".cyan().bold(),
        report.access1.thread_id
    );
    eprintln!("  Type: {:?}", report.access1.access_type);
    eprintln!("  Lock Held: {:?}", report.access1.lock_held);
    eprintln!("  {}", "Backtrace:".magenta());
    for (j, frame) in report.access1.backtrace.iter().take(4).enumerate() {
        eprintln!("    {} {}", format!("{}.", j + 1).dimmed(), frame.dimmed());
    }

    eprintln!(
        "\n{} (Thread {})",
        "Access 2:".cyan().bold(),
        report.access2.thread_id
    );
    eprintln!("  Type: {:?}", report.access2.access_type);
    eprintln!("  Lock Held: {:?}", report.access2.lock_held);
    eprintln!("  {}", "Backtrace:".magenta());
    for (j, frame) in report.access2.backtrace.iter().take(4).enumerate() {
        eprintln!("    {} {}", format!("{}.", j + 1).dimmed(), frame.dimmed());
    }
    eprintln!();
}

pub fn print_report() {
    let count = ISSUE_COUNT.load(Ordering::Relaxed);
    if count > 0 {
        eprintln!(
            "\n{} Thread Sentry detected {} issue(s)\n",
            "⚠️".bright_red(),
            count.to_string().bright_red().bold()
        );
    } else {
        eprintln!(
            "\n{} No issues detected by Thread Sentry\n",
            "✓".bright_green()
        );
    }
}
