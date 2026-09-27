use std::time::Instant;

use encase::ShaderType;
use tufa::{
    export::{
        nalgebra::Vector2,
        winit::{
            event::{ElementState, KeyEvent},
            keyboard::{KeyCode, PhysicalKey},
        },
    },
    prelude::*,
};

use crate::pages::Layout;

pub struct App {
    pub start: Instant,
    pub uniform: Uniform,
    pub texture_count: u32,

    pub render: RenderPipeline,
    pub vertex: VertexBuffer<Vertex>,
    pub index: IndexBuffer,
}

#[derive(Default, ShaderType)]
pub struct Uniform {
    pub viewport: Vector2<f32>,
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,

    pub n: u32,
}

impl Interactive for App {
    fn init(&mut self, _gcx: GraphicsCtx) {}

    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let t = self.start.elapsed().as_secs_f32();

        let size = gcx.window.inner_size();
        let scale_factor = gcx.window.scale_factor() as f32;
        let viewport = Vector2::new(size.width, size.height).cast();
        let mut layout = Layout::new(viewport);

        let size = Vector2::new(1404, 1872).cast::<f32>();
        let aspect = Vector2::new(1.0, size.y / size.x);

        let nx = 3;
        let gap = 20.0 * scale_factor;

        let page_width = (viewport.x - gap * (nx + 1) as f32) / nx as f32;
        let page = Vector2::new(page_width, aspect.y * page_width);
        let ny = (viewport.y / page.y).ceil() as u32 + 2;
        let size = aspect * page.x;

        for i in 0..nx {
            let direction = (1 - 2 * (i % 2 == 0) as i8) as f32;
            let (step, scroll) = (page.y + gap, t * 300.0 * direction);
            let (animation, iteration) = (scroll % step, (scroll / step) as i32);

            for j in -1..ny as i32 {
                let center =
                    Vector2::new((page.x + gap) * i as f32 + gap, step * j as f32 - animation);

                let texture = ((iteration + j) * nx as i32 + i as i32) % self.texture_count as i32;
                layout.add_page(texture as u32 % self.texture_count, center, size);
            }
        }

        let (vertex, index) = layout.into_inner();
        let idx_count = index.len() as u32;
        self.vertex.upload(&vertex);
        self.index.upload(&index);

        self.render
            .draw(render_pass, &self.index, &self.vertex, 0..idx_count);
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
