use std::sync::Mutex;

use crate::glfw_input::GlfwInputs;

mod glfw_input_listener_tests;

struct FakeGlfwInputs {
    fake_key_actions: Mutex<Vec<glfw::Key>>,
}

impl FakeGlfwInputs {
    fn new(fake_key_actions: Vec<glfw::Key>) -> Self {
        let fake_key_actions = Mutex::new(fake_key_actions);
        Self { fake_key_actions }
    }
}
impl GlfwInputs for FakeGlfwInputs {
    fn get_key(&self, key: glfw::Key) -> glfw::Action {
        let mut key_actions = self.fake_key_actions.lock().unwrap();
        let index = key_actions.iter().position(|ka| ka == &key);
        if let Some(index) = index {
            key_actions.remove(index);
            glfw::Action::Press
        } else {
            glfw::Action::Release
        }
    }

    fn on_cursor_move(&self, fun: Box<dyn Fn(crate::math::Vec2)>) {}

    fn on_scroll(&self, fun: Box<dyn Fn(crate::math::Vec2)>) {}
}
