//! CLI for the bike asset pipeline.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use velo_bikegen::{
    default_bikes_dir, import_bike_from_images, list_bikes, load_bike_asset, BikeImportError,
};

#[derive(Parser)]
#[command(name = "velo-bikegen", about = "VeloSim bike image-to-3D asset pipeline (M3c)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import 1–4 images into the bike library.
    Import {
        /// Source image paths (1–4).
        #[arg(required = true, value_name = "IMAGE")]
        images: Vec<PathBuf>,
        /// Bike library id (folder name).
        #[arg(long)]
        id: String,
        /// Display name.
        #[arg(long)]
        name: Option<String>,
        /// Bike library root (default: ~/Documents/VeloSim/bikes).
        #[arg(long)]
        bikes_dir: Option<PathBuf>,
    },
    /// List bikes in the library.
    List {
        #[arg(long)]
        bikes_dir: Option<PathBuf>,
    },
    /// Show paths for a bike id.
    Show {
        id: String,
        #[arg(long)]
        bikes_dir: Option<PathBuf>,
    },
}

fn main() -> Result<(), BikeImportError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Import {
            images,
            id,
            name,
            bikes_dir,
        } => {
            let dir = bikes_dir.unwrap_or_else(default_bikes_dir);
            let asset = import_bike_from_images(&dir, &images, &id, name.as_deref())?;
            println!("Imported bike '{}' → {}", asset.bike_id, asset.gltf_path.display());
        }
        Commands::List { bikes_dir } => {
            let dir = bikes_dir.unwrap_or_else(default_bikes_dir);
            for bike in list_bikes(&dir)? {
                println!("{}\t{}", bike.bike_id, bike.name);
            }
        }
        Commands::Show { id, bikes_dir } => {
            let dir = bikes_dir.unwrap_or_else(default_bikes_dir);
            let asset = load_bike_asset(&dir, &id)?;
            println!("id: {}", asset.bike_id);
            println!("gltf: {}", asset.gltf_path.display());
            println!(
                "anchor: translation={:?} rotation_y={} scale={}",
                asset.anchor.translation, asset.anchor.rotation_y, asset.anchor.scale
            );
        }
    }
    Ok(())
}
