use std::sync::{Arc, Mutex};

use crate::*;
use crate::{Clock, math};

#[cfg(test)]
mod tests;

pub struct GlfwInputListener {
    glfw_inputs: Box<dyn GlfwInputs>,
    cursor_offset: Arc<Mutex<Option<math::Vec2>>>,
    scroll: Arc<Mutex<f32>>,
}
impl GlfwInputListener {
    pub fn init(glfw_inputs: Box<dyn GlfwInputs>) -> Self {
        let cursor_offset = Self::setup_cursor_callback(glfw_inputs.as_ref());
        let scroll = Self::setup_scroll_callback(glfw_inputs.as_ref());
        Self {
            glfw_inputs,
            cursor_offset,
            scroll,
        }
    }
    fn setup_cursor_callback(glfw_inputs: &dyn GlfwInputs) -> Arc<Mutex<Option<math::Vec2>>> {
        let shared_offset = Arc::new(Mutex::new(Option::<math::Vec2>::None));
        let cursor_offset = shared_offset.clone();
        let shared_position = Arc::new(Mutex::new(vec2!(0.0)));
        let cursor_position = shared_position.clone();
        let update_cursor_offset = move |position: math::Vec2| {
            let sensitivity = 0.1;
            let mut cursor_offset = cursor_offset.lock().unwrap();
            *cursor_offset = Some(vec2!(0.0));
            let mut cursor_position = cursor_position.lock().unwrap();
            let x = position.x;
            let y = position.y;
            let offset_x = x - cursor_position.x;
            let offset_y = cursor_position.y - y;
            cursor_position.x = x;
            cursor_position.y = y;
            let cursor_offset = cursor_offset.as_mut().unwrap();
            cursor_offset.x += offset_x * sensitivity;
            cursor_offset.y = (cursor_offset.y + offset_y * sensitivity).clamp(-89.0, 89.0);
        };
        glfw_inputs.on_cursor_move(Box::new(update_cursor_offset));
        shared_offset
    }

    fn setup_scroll_callback(glfw_inputs: &dyn GlfwInputs) -> Arc<Mutex<f32>> {
        let shared_scroll = Arc::new(Mutex::new(0.0));
        let scroll = shared_scroll.clone();
        let update_scroll = move |position: math::Vec2| {
            let sensitivity = 10.0;
            let mut scroll = scroll.lock().unwrap();
            *scroll += sensitivity * position.y;
        };
        glfw_inputs.on_scroll(Box::new(update_scroll));
        shared_scroll
    }

    fn get_cursor_offset(&self) -> Option<math::Vec2> {
        let offset = self.cursor_offset.lock().unwrap();
        offset.clone()
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
        let glfw_inputs = &self.glfw_inputs;
        let mut inputs = vec![];
        if glfw_inputs.get_key(glfw::Key::W) == glfw::Action::Press {
            inputs.push(Input::Forward);
        }
        if glfw_inputs.get_key(glfw::Key::S) == glfw::Action::Press {
            inputs.push(Input::Backwards);
        }
        if glfw_inputs.get_key(glfw::Key::D) == glfw::Action::Press {
            inputs.push(Input::Right);
        }
        if glfw_inputs.get_key(glfw::Key::A) == glfw::Action::Press {
            inputs.push(Input::Left);
        }
        if let Some(cursor_offset) = self.get_cursor_offset() {
            inputs.push(Input::MoveCursor(cursor_offset));
        }
        if let Some(scroll) = self.get_scroll_offset() {
            inputs.push(Input::Scroll(scroll));
        }
        inputs
    }
}

pub trait GlfwInputs {
    fn get_key(&self, key: glfw::Key) -> glfw::Action;
    fn on_cursor_move(&self, fun: Box<dyn Fn(math::Vec2)>);
    fn on_scroll(&self, fun: Box<dyn Fn(math::Vec2)>);
}
pub struct GlfwInputsImpl {
    window: Arc<Mutex<glfw::PWindow>>,
}
impl GlfwInputsImpl {
    pub fn new(window: Arc<Mutex<glfw::PWindow>>) -> Self {
        Self { window }
    }
}
impl GlfwInputs for GlfwInputsImpl {
    fn get_key(&self, key: glfw::Key) -> glfw::Action {
        let window = self.window.lock().unwrap();
        window.get_key(key)
    }
    fn on_cursor_move(&self, fun: Box<dyn Fn(math::Vec2)>) {
        let mut window = self.window.lock().unwrap();
        window.set_cursor_pos_callback(move |_, x, y| fun(vec2!(x as f32, y as f32)));
    }
    fn on_scroll(&self, fun: Box<dyn Fn(math::Vec2)>) {
        let mut window = self.window.lock().unwrap();
        window.set_scroll_callback(move |_, x, y| fun(vec2!(x as f32, y as f32)));
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
