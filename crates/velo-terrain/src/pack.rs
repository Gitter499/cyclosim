use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use png::{Encoder, ColorType, BitDepth};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::mesh::TerrainMesh;

pub const TERRAIN_MESH_FILE: &str = "terrain.mesh.bin";
pub const TERRAIN_TEXTURE_FILE: &str = "terrain.png";
pub const TERRAIN_META_FILE: &str = "terrain.json";

const MESH_MAGIC: &[u8; 4] = b"VTM1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMeta {
    pub texture_width: u32,
    pub texture_height: u32,
    pub vertex_count: u32,
    pub index_count: u32,
}

#[derive(Debug, Error)]
pub enum TerrainPackError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("png: {0}")]
    Png(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid mesh file")]
    InvalidMesh,
    #[error("pack missing {0}")]
    MissingFile(String),
}

#[derive(Debug, Clone)]
pub struct TerrainPack {
    pub mesh: TerrainMesh,
    pub texture_rgba: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,
}

impl TerrainPack {
    pub fn write_to_dir(&self, dir: &Path) -> Result<(), TerrainPackError> {
        fs::create_dir_all(dir)?;
        write_mesh_binary(&dir.join(TERRAIN_MESH_FILE), &self.mesh)?;
        write_texture_png(
            &dir.join(TERRAIN_TEXTURE_FILE),
            self.texture_width,
            self.texture_height,
            &self.texture_rgba,
        )?;
        let meta = TerrainMeta {
            texture_width: self.texture_width,
            texture_height: self.texture_height,
            vertex_count: self.mesh.vertices.len() as u32,
            index_count: self.mesh.indices.len() as u32,
        };
        fs::write(
            dir.join(TERRAIN_META_FILE),
            serde_json::to_vec_pretty(&meta)?,
        )?;
        Ok(())
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self, TerrainPackError> {
        let mesh_path = dir.join(TERRAIN_MESH_FILE);
        if !mesh_path.exists() {
            return Err(TerrainPackError::MissingFile(TERRAIN_MESH_FILE.into()));
        }
        let mesh = read_mesh_binary(&mesh_path)?;
        let tex_path = dir.join(TERRAIN_TEXTURE_FILE);
        if !tex_path.exists() {
            return Err(TerrainPackError::MissingFile(TERRAIN_TEXTURE_FILE.into()));
        }
        let (texture_rgba, texture_width, texture_height) = read_texture_png(&tex_path)?;
        Ok(Self {
            mesh,
            texture_rgba,
            texture_width,
            texture_height,
        })
    }
}

fn write_mesh_binary(path: &Path, mesh: &TerrainMesh) -> Result<(), TerrainPackError> {
    let mut buf = Vec::new();
    buf.extend_from_slice(MESH_MAGIC);
    buf.extend_from_slice(&(mesh.vertices.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(mesh.indices.len() as u32).to_le_bytes());
    for v in &mesh.vertices {
        buf.extend_from_slice(&v.position[0].to_le_bytes());
        buf.extend_from_slice(&v.position[1].to_le_bytes());
        buf.extend_from_slice(&v.position[2].to_le_bytes());
        buf.extend_from_slice(&v.uv[0].to_le_bytes());
        buf.extend_from_slice(&v.uv[1].to_le_bytes());
    }
    for idx in &mesh.indices {
        buf.extend_from_slice(&idx.to_le_bytes());
    }
    let mut f = fs::File::create(path)?;
    f.write_all(&buf)?;
    Ok(())
}

fn read_mesh_binary(path: &Path) -> Result<TerrainMesh, TerrainPackError> {
    let mut f = fs::File::open(path)?;
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic)?;
    if &magic != MESH_MAGIC {
        return Err(TerrainPackError::InvalidMesh);
    }
    let mut u32buf = [0u8; 4];
    f.read_exact(&mut u32buf)?;
    let vcount = u32::from_le_bytes(u32buf) as usize;
    f.read_exact(&mut u32buf)?;
    let icount = u32::from_le_bytes(u32buf) as usize;

    let mut vertices = Vec::with_capacity(vcount);
    for _ in 0..vcount {
        let mut f32buf = [0u8; 4];
        let mut pos = [0f32; 3];
        for p in &mut pos {
            f.read_exact(&mut f32buf)?;
            *p = f32::from_le_bytes(f32buf);
        }
        let mut uv = [0f32; 2];
        for u in &mut uv {
            f.read_exact(&mut f32buf)?;
            *u = f32::from_le_bytes(f32buf);
        }
        vertices.push(crate::TerrainVertex { position: pos, uv });
    }

    let mut indices = Vec::with_capacity(icount);
    for _ in 0..icount {
        f.read_exact(&mut u32buf)?;
        indices.push(u32::from_le_bytes(u32buf));
    }

    Ok(TerrainMesh { vertices, indices })
}

fn write_texture_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), TerrainPackError> {
    let mut encoder = Encoder::new(fs::File::create(path)?, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| TerrainPackError::Png(e.to_string()))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| TerrainPackError::Png(e.to_string()))?;
    Ok(())
}

fn read_texture_png(path: &Path) -> Result<(Vec<u8>, u32, u32), TerrainPackError> {
    let decoder = png::Decoder::new(fs::File::open(path)?);
    let mut reader = decoder
        .read_info()
        .map_err(|e| TerrainPackError::Png(e.to_string()))?;
    let width = reader.info().width;
    let height = reader.info().height;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader
        .next_frame(&mut buf)
        .map_err(|e| TerrainPackError::Png(e.to_string()))?;
    Ok((buf, width, height))
}
