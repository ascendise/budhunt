use std::sync::{Arc, Mutex};

use crate::gfx::*;

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
            .unwrap_or_else(|| panic!("Frame {idx} was not rendered!"))
            .clone()
    }
}
impl Renderer for SpyRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, models: &[Model], lights: &[Light]) {
        let frame = Frame {
            projection: projection.clone(),
            camera: camera.clone(),
            models: models.to_vec(),
            lights: lights.to_vec(),
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
