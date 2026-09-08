use std::sync::{Arc, Mutex};

use crate::gfx::*;

mod gltf_tests;
mod ibl_tests;
mod render_system_tests;

#[derive(Clone)]
struct SpyRenderer {
    frames: Arc<Mutex<Vec<Frame>>>,
}
impl SpyRenderer {
    pub fn new() -> Self {
        Self {
            frames: Default::default(),
        }
    }

    // Fails test if frame does not exist
    pub fn frame(&self, idx: usize) -> Frame {
        let frames = &self.frames.lock().unwrap();
        frames
            .get(idx)
            .expect("Frame {idx} was not rendered!")
            .clone()
    }
}
impl Renderer for SpyRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, renderables: &[Renderable]) {
        let frame = Frame {
            projection: projection.clone(),
            camera: camera.clone(),
            models: renderables
                .iter()
                .filter_map(|m| maybe_component!(m, Renderable::Model))
                .cloned()
                .collect(),
            lights: renderables
                .iter()
                .filter_map(|m| maybe_component!(m, Renderable::Light))
                .cloned()
                .collect(),
        };
        let mut frames = self.frames.lock().unwrap();
        frames.push(frame);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Frame {
    projection: Projection,
    camera: Camera,
    models: Vec<Model>,
    lights: Vec<Light>,
}
