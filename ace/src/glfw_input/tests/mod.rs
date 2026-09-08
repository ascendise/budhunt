use std::sync::Mutex;

use crate::{glfw_input::GlfwInputs, math};

mod glfw_input_listener_tests;

struct FakeGlfwInputs {
    key_actions: Mutex<Vec<glfw::Key>>,
    mouse_actions: Mutex<Vec<glfw::MouseButton>>,
    cursor_actions: Mutex<Vec<math::Vec2>>,
    scroll_actions: Mutex<Vec<math::Vec2>>,
}

impl FakeGlfwInputs {
    fn new() -> Self {
        Self {
            key_actions: Mutex::new(vec![]),
            mouse_actions: Mutex::new(vec![]),
            cursor_actions: Mutex::new(vec![]),
            scroll_actions: Mutex::new(vec![]),
        }
    }

    fn key_actions(&self, key_actions: Vec<glfw::Key>) {
        *self.key_actions.lock().unwrap() = key_actions;
    }

    fn mouse_actions(&self, mouse_actions: Vec<glfw::MouseButton>) {
        *self.mouse_actions.lock().unwrap() = mouse_actions;
    }

    /// Define which position the cursor moved to (top-left orientation)
    fn cursor_actions(&self, cursor_actions: Vec<math::Vec2>) {
        *self.cursor_actions.lock().unwrap() = cursor_actions;
    }

    fn scroll_actions(&self, scroll_actions: Vec<math::Vec2>) {
        *self.scroll_actions.lock().unwrap() = scroll_actions;
    }
}
impl GlfwInputs for FakeGlfwInputs {
    fn get_key(&self, key: glfw::Key) -> glfw::Action {
        let mut key_actions = self.key_actions.lock().unwrap();
        let index = key_actions.iter().position(|ka| ka == &key);
        if let Some(index) = index {
            key_actions.remove(index);
            glfw::Action::Press
        } else {
            glfw::Action::Release
        }
    }

    fn on_cursor_move(&self, fun: Box<dyn Fn(crate::math::Vec2)>) {
        for cursor_action in self.cursor_actions.lock().unwrap().drain(..) {
            fun(cursor_action);
        }
    }

    fn on_scroll(&self, fun: Box<dyn Fn(crate::math::Vec2)>) {
        for scroll_action in self.scroll_actions.lock().unwrap().drain(..) {
            fun(scroll_action);
        }
    }

    fn get_mouse_button(&self, mouse_button: glfw::MouseButton) -> glfw::Action {
        let mut mouse_actions = self.mouse_actions.lock().unwrap();
        let index = mouse_actions.iter().position(|ma| ma == &mouse_button);
        if let Some(index) = index {
            mouse_actions.remove(index);
            glfw::Action::Press
        } else {
            glfw::Action::Release
        }
    }
}
