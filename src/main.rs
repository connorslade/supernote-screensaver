use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::Instant,
};

use anyhow::Result;
use encase::ShaderType;
use image::{EncodableLayout, ImageReader};
use tufa::{
    bindings::{collection::texture_collection::TextureCollection, texture::format::Rf8},
    export::{
        nalgebra::Vector2,
        wgpu::{Features, FilterMode, Limits},
        winit::{
            event::{ElementState, KeyEvent},
            keyboard::{KeyCode, PhysicalKey},
        },
    },
    prelude::*,
};

fn main() -> Result<()> {
    let gpu = Gpu::builder()
        .with_features(Features::TEXTURE_BINDING_ARRAY | Features::PUSH_CONSTANTS)
        .with_limits(Limits {
            max_binding_array_elements_per_shader_stage: 500_000,
            max_push_constant_size: 4,
            ..Default::default()
        })
        .build()?;
    let textures = Textures::load(&gpu, PathBuf::from("assets/png-1x"))?;

    let sampler = gpu.create_sampler(FilterMode::Linear);
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .push_constants(4)
        .bind(&textures.collection, ShaderStages::FRAGMENT)
        .bind(&sampler, ShaderStages::FRAGMENT)
        .finish();

    let start = Instant::now();
    let uniform = Uniform { n: 0 };
    gpu.create_window(
        WindowAttributes::default().with_title("Supernote Screensaver"),
        App {
            start,
            uniform,
            render,
        },
    )
    .run()?;

    Ok(())
}

struct App {
    start: Instant,
    uniform: Uniform,

    render: RenderPipeline,
}

#[derive(Default, ShaderType)]
struct Uniform {
    n: u32,
}

struct Textures {
    _textures: Vec<Texture<Rf8>>,
    collection: TextureCollection,
}

impl Interactive for App {
    fn init(&mut self, _gcx: GraphicsCtx) {}

    fn render(&mut self, _gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        self.render
            .set_push_constants(u32::to_ne_bytes(self.uniform.n).to_vec());
        self.render.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, _ctx: &Context) {}

    fn window_event(&mut self, _gcx: GraphicsCtx, _event: &WindowEvent) {
        match _event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match code {
                KeyCode::ArrowRight => self.uniform.n += 1,
                KeyCode::ArrowLeft => self.uniform.n -= 1,
                _ => {}
            },
            _ => {}
        }
    }

    fn device_event(&mut self, _gcx: GraphicsCtx, _device_id: DeviceId, _event: &DeviceEvent) {}
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

        let collection = gpu.create_texture_collection(&textures);

        Ok(Self {
            _textures: textures,
            collection,
        })
    }
}
