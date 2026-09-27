use std::{path::PathBuf, time::Instant};

use anyhow::Result;
use encase::ShaderSize;
use tufa::{
    export::wgpu::{Features, FilterMode, Limits},
    prelude::*,
};

use crate::{
    app::{App, Uniform},
    pages::Textures,
};

mod app;
mod pages;

fn main() -> Result<()> {
    let gpu = Gpu::builder()
        .with_features(
            Features::TEXTURE_BINDING_ARRAY
                | Features::PUSH_CONSTANTS
                | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        )
        .with_limits(Limits {
            max_binding_array_elements_per_shader_stage: 500_000,
            max_push_constant_size: Uniform::SHADER_SIZE.get() as u32,
            ..Default::default()
        })
        .build()?;
    let textures = Textures::load(&gpu, PathBuf::from("assets/png-1x"))?;

    let sampler = gpu.create_sampler(FilterMode::Linear);
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .push_constants(Uniform::SHADER_SIZE.get() as u32)
        .bind(&textures.collection, ShaderStages::FRAGMENT)
        .bind(&sampler, ShaderStages::FRAGMENT)
        .finish();

    let vertex = gpu.create_vertex_empty(0);
    let index = gpu.create_index_empty(0);
    let start = Instant::now();
    let uniform = Uniform {
        n: 0,
        ..Default::default()
    };

    let app = App {
        start,
        uniform,
        render,
        texture_count: textures.textures.len() as u32,
        index,
        vertex,
    };

    gpu.create_window(
        WindowAttributes::default().with_title("Supernote Screensaver"),
        app,
    )
    .run()?;

    Ok(())
}
