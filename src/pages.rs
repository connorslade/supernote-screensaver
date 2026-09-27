use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use anyhow::Result;
use image::{EncodableLayout, ImageReader};
use rand::{rng, seq::SliceRandom};
use tufa::{
    bindings::{collection::texture_collection::TextureCollection, texture::format::Rf8},
    export::nalgebra::Vector2,
    pipeline::render::consts::{QUAD_INDEX, QUAD_VERTEX},
    prelude::*,
};

pub struct Textures {
    pub textures: Vec<Texture<Rf8>>,
    pub collection: TextureCollection,
}

pub struct Layout {
    viewport: Vector2<f32>,

    vertex: Vec<Vertex>,
    index: Vec<u32>,
}

impl Textures {
    pub fn load(gpu: &Gpu, path: PathBuf) -> Result<Self> {
        let mut textures = Vec::new();

        for entry in fs::read_dir(path)? {
            let image = ImageReader::new(BufReader::new(File::open(dbg!(entry?.path()))?))
                .with_guessed_format()?
                .decode()?;

            let texture = gpu.create_texture_2d::<Rf8>(Vector2::new(image.width(), image.height()));
            texture.upload(image.to_luma8().as_bytes());
            textures.push(texture);
        }

        textures.shuffle(&mut rng());
        let collection = gpu.create_texture_collection(&textures);

        Ok(Self {
            textures,
            collection,
        })
    }
}

impl Layout {
    pub fn new(viewport: Vector2<f32>) -> Self {
        Self {
            viewport,
            vertex: Vec::new(),
            index: Vec::new(),
        }
    }

    pub fn into_inner(self) -> (Vec<Vertex>, Vec<u32>) {
        (self.vertex, self.index)
    }

    pub fn add_page(&mut self, texture: u32, position: Vector2<f32>, size: Vector2<f32>) {
        let uv_offset = Vector2::repeat(texture as f32);
        let half = size / 2.0;
        let scale = half.component_div(&self.viewport);
        let translate = (position + half).component_div(&self.viewport);

        let b = self.vertex.len() as u32;
        for vert in QUAD_VERTEX {
            let position = vert.position.xy().component_mul(&scale) + translate;
            let clip = position * 2.0 - Vector2::repeat(1.0);
            let vertex = Vertex::new(clip.push(0.0).push(1.0), uv_offset + vert.uv);
            self.vertex.push(vertex);
        }

        QUAD_INDEX.iter().for_each(|idx| self.index.push(b + idx));
    }
}
