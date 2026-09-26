use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::Instant,
};

use anyhow::Result;
use encase::{ShaderSize, ShaderType};
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

    let start = Instant::now();
    let uniform = Uniform {
        n: 0,
        ..Default::default()
    };

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
    viewport: Vector2<f32>,
    position: Vector2<f32>,
    size: Vector2<f32>,

    n: u32,
}

struct Textures {
    _textures: Vec<Texture<Rf8>>,
    collection: TextureCollection,
}

impl Interactive for App {
    fn init(&mut self, _gcx: GraphicsCtx) {}

    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let t = self.start.elapsed().as_secs_f32();

        let gap = 50.0;

        for i in 0..3 {
            for j in 0..3 {
                let size = gcx.window.inner_size();
                let viewport = Vector2::new(size.width, size.height).cast();
                let size = Vector2::new(1404, 1872).cast();
                self.uniform.n += i + 3 * j;
                self.uniform.viewport = viewport;
                self.uniform.position = size
                    + Vector2::new(0.0, -t * (1 - 2 * (j % 2 == 0) as i8) as f32 * 500.0)
                    + Vector2::new(
                        2.0 * (size.x + gap) * j as f32,
                        2.0 * (size.y + gap) * i as f32,
                    );
                self.uniform.size = size;

                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.uniform).unwrap();
                self.render.set_push_constants(buffer.into_inner());
                self.uniform.n -= i + 3 * j;

                self.render.draw_quad(render_pass, 0..1);
            }
        }
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
