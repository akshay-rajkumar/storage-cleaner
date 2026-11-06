use clap::{Parser, Subcommand};
use cleaner_core::{run_full_scan, FullScanResult};

#[derive(Parser)]
#[command(name = "cleaner", about = "Linux Cleaner — read-only storage scanner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan the system for cleanable items
    Scan {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { json } => {
            let result = run_full_scan()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_table(&result);
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;

    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn print_table(result: &FullScanResult) {
    let du = &result.disk_usage;
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Linux Cleaner — Scan                     ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Disk Usage ({}):", du.mount_point);
    println!("║    Total: {:>12}    Used: {:>12}    Free: {:>12}",
        format_size(du.total_bytes),
        format_size(du.used_bytes),
        format_size(du.free_bytes),
    );

    let pct = if du.total_bytes > 0 {
        (du.used_bytes as f64 / du.total_bytes as f64 * 100.0) as u32
    } else {
        0
    };
    let bar_width = 40;
    let filled = (bar_width * pct / 100) as usize;
    let empty = bar_width as usize - filled;
    println!("║    [{}{}] {}%", "█".repeat(filled), "░".repeat(empty), pct);
    println!("╠══════════════════════════════════════════════════════════════╣");

    for cat in &result.categories {
        println!("║");
        println!("║  📁 {}  ({})", cat.category, format_size(cat.total_bytes));
        println!("║  {:<35} {:>10}  {:>8}", "Name", "Size", "Risk");
        println!("║  {}", "─".repeat(56));

        for item in &cat.items {
            let size = format_size(item.size_bytes);
            let risk = format!("{}", item.risk);
            let label = if item.label.len() > 35 {
                format!("{}…", &item.label[..34])
            } else {
                item.label.clone()
            };
            println!("║  {:<35} {:>10}  {:>8}", label, size, risk);
        }
    }

    println!("║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}
