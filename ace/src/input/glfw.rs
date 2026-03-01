use std::sync::{Arc, Mutex};

use crate::*;
use crate::{
    Clock,
    input::{Input, InputListener},
    math,
};

pub struct GlfwInputListener {
    window: Arc<Mutex<glfw::PWindow>>,
    cursor_offset: Arc<Mutex<math::Vec2>>,
    scroll: Arc<Mutex<f32>>,
}
impl GlfwInputListener {
    pub fn init(window: Arc<Mutex<glfw::PWindow>>) -> Self {
        let mut win = window.lock().unwrap();
        let cursor_offset = Self::setup_cursor_callback(&mut win);
        let scroll = Self::setup_scroll_callback(&mut win);
        drop(win);
        Self {
            window,
            cursor_offset,
            scroll,
        }
    }

    fn setup_cursor_callback(window: &mut glfw::PWindow) -> Arc<Mutex<math::Vec2>> {
        let shared_offset = Arc::new(Mutex::new(vec2!(0.0)));
        let cursor_offset = shared_offset.clone();
        let shared_position = Arc::new(Mutex::new(vec2!(0.0)));
        let cursor_position = shared_position.clone();
        window.set_cursor_pos_callback(move |_, x, y| {
            let sensitivity = 0.1;
            let mut cursor_offset = cursor_offset.lock().unwrap();
            let mut cursor_position = cursor_position.lock().unwrap();
            let x = x as f32;
            let y = y as f32;
            let offset_x = x - cursor_position.x;
            let offset_y = cursor_position.y - y;
            cursor_position.x = x;
            cursor_position.y = y;
            cursor_offset.x += offset_x * sensitivity;
            cursor_offset.y = (cursor_offset.y + offset_y * sensitivity).clamp(-89.0, 89.0);
        });
        shared_offset
    }

    fn setup_scroll_callback(window: &mut glfw::PWindow) -> Arc<Mutex<f32>> {
        let shared_scroll = Arc::new(Mutex::new(0.0));
        let scroll = shared_scroll.clone();
        window.set_scroll_callback(move |_, _, y| {
            let sensitivity = 10.0;
            let y = y as f32;
            let mut scroll = scroll.lock().unwrap();
            *scroll += sensitivity * y;
        });
        shared_scroll
    }

    fn get_scroll_offset(&self) -> Option<f32> {
        let mut scroll = self.scroll.lock().unwrap();
        let s = *scroll;
        *scroll = 0.0;
        if s == 0.0 { None } else { Some(s) }
    }
}

impl InputListener for GlfwInputListener {
    fn get_inputs(&self) -> Vec<Input> {
        let window = &self.window.lock().unwrap();
        let mut inputs = vec![];
        if window.get_key(glfw::Key::W) == glfw::Action::Press {
            inputs.push(Input::Forward);
        }
        if window.get_key(glfw::Key::S) == glfw::Action::Press {
            inputs.push(Input::Backwards);
        }
        if window.get_key(glfw::Key::D) == glfw::Action::Press {
            inputs.push(Input::Right);
        }
        if window.get_key(glfw::Key::A) == glfw::Action::Press {
            inputs.push(Input::Left);
        }
        let cursor_offset = Input::MoveCursor(self.get_cursor_offset());
        inputs.push(cursor_offset);
        if let Some(scroll) = self.get_scroll_offset() {
            inputs.push(Input::Scroll(scroll));
        }
        inputs
    }

    fn get_cursor_offset(&self) -> math::Vec2 {
        let offset = self.cursor_offset.lock().unwrap();
        offset.clone()
    }
}

#[derive(Debug, Clone)]
pub struct GlfwClock {
    glfw: glfw::Glfw,
    time_frame: Arc<Mutex<Timeframe>>,
}
impl GlfwClock {
    pub fn new(glfw: glfw::Glfw) -> Self {
        Self {
            glfw,
            time_frame: Arc::new(Mutex::new(Default::default())),
        }
    }
}
impl Clock for GlfwClock {
    fn time_delta(&self) -> f32 {
        self.time_frame.lock().unwrap().delta
    }

    fn stop_frame_time(&self) {
        let mut time_frame = self.time_frame.lock().unwrap();
        let now = self.glfw.get_time();
        let delta = now - time_frame.last_frame;
        time_frame.delta = delta as f32;
        time_frame.last_frame = now;
    }
}
#[derive(Debug, Clone, Default)]
struct Timeframe {
    delta: f32,
    last_frame: f64,
}
