use std::path::PathBuf;

use clap::Parser;
use velo_terrain::{bake_terrain_pack, DEFAULT_CELL_M, DEFAULT_CORRIDOR_M};

#[derive(Parser)]
#[command(name = "velo-terrain", about = "Bake terrain mesh + texture into route pack")]
struct Args {
    /// Route pack directory (must contain route.json)
    #[arg(short, long)]
    pack: PathBuf,

    /// Corridor half-width in meters
    #[arg(long, default_value_t = DEFAULT_CORRIDOR_M)]
    corridor_m: f64,

    /// DEM cell size in meters
    #[arg(long, default_value_t = DEFAULT_CELL_M)]
    cell_m: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mesh = bake_terrain_pack(&args.pack, args.corridor_m, args.cell_m)?;
    eprintln!(
        "Baked terrain into {:?}: {} vertices, {} indices",
        args.pack,
        mesh.vertices.len(),
        mesh.indices.len()
    );
    Ok(())
}
