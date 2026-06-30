use std::path::PathBuf;

use clap::Parser;
use velo_route_import::{import_file, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};

#[derive(Parser)]
#[command(name = "velo-route-import", about = "Import GPX/TCX → route pack")]
struct Args {
    /// Input GPX/TCX file
    #[arg(short, long)]
    input: PathBuf,

    /// Route pack output directory
    #[arg(short, long)]
    output: PathBuf,

    /// Route identifier (directory name)
    #[arg(short, long)]
    route_id: String,

    /// Display name
    #[arg(short, long)]
    name: Option<String>,

    /// Resample spacing in meters
    #[arg(long, default_value_t = DEFAULT_SPACING_M)]
    spacing_m: f64,

    /// Grade smoothing window in meters
    #[arg(long, default_value_t = DEFAULT_GRADE_WINDOW_M)]
    grade_window_m: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let model = import_file(
        &args.input,
        &args.route_id,
        args.name.as_deref(),
        args.spacing_m,
        args.grade_window_m,
    )?;
    std::fs::create_dir_all(&args.output)?;
    model.save_pack(&args.output)?;
    eprintln!(
        "Wrote route pack {:?}: {} points, {:.0} m",
        args.output,
        model.points.len(),
        model.total_distance_m()
    );
    Ok(())
}
