use crate::*;

pub mod glfw;
#[cfg(test)]
mod tests;

pub struct InputSystem {
    clock: Box<dyn Clock>,
}
impl System for InputSystem {
    fn run(&self, entities: &mut Entities, inputs: &[Input]) {
        let player = entities.get_entity(Entities::PLAYER_IDX);
        let camera = player
            .iter()
            .find(|c| matches!(c, Component::Position(_)))
            .expect("No camera position found");
        let mut camera = component!(camera, Component::Position).clone();
        let cursor_offset = inputs
            .iter()
            .find(|i| matches!(i, Input::MoveCursor(_)))
            .map(|i| component!(i, Input::MoveCursor).clone())
            .unwrap_or(vec2!(0.0));
        let move_dir = self.turn_camera(&mut camera, &cursor_offset);
        self.move_camera(&mut camera, inputs, &move_dir);
        entities.update_entity(0, Component::Position(camera));
    }
}

impl InputSystem {
    pub fn new(clock: Box<dyn Clock>) -> Self {
        Self { clock }
    }

    /// Moves camera on xyz-axis and returns movement direction (y axis rotation)
    fn turn_camera(&self, camera: &mut Position, offset: &math::Vec2) -> math::Vec3 {
        let yaw = math::radians(offset.x);
        let pitch = math::radians(offset.y);
        let move_dir = math::Vec3 {
            x: yaw.cos(),
            y: 0.0,
            z: yaw.sin(),
        };
        let move_dir = move_dir.normalize();
        let turn_dir = math::Vec3 {
            x: yaw.cos() * pitch.cos(),
            y: pitch.sin(),
            z: yaw.sin() * pitch.cos(),
        }
        .normalize();
        camera.direction = turn_dir;
        move_dir
    }

    fn move_camera(&self, camera: &mut Position, inputs: &[Input], move_direction: &math::Vec3) {
        let mut movement = math::Vec3::default();
        let speed = 10.0;
        let speed = self.clock.time_delta() * speed;
        let front = move_direction.normalize();
        let up = vec3!(0.0, 1.0, 0.0);
        let strafe = front.cross(&up).normalize();
        if inputs.contains(&Input::Forward) {
            movement = &movement + &front;
        }
        if inputs.contains(&Input::Backwards) {
            movement = &movement - &front;
        }
        if inputs.contains(&Input::Right) {
            movement = &(&movement / 2.0) + &strafe;
        }
        if inputs.contains(&Input::Left) {
            movement = &(&movement / 2.0) - &strafe;
        }
        let movement = movement * speed;
        camera.position = &camera.position + &movement;
    }
}

pub trait InputListener {
    fn get_inputs(&self) -> Vec<Input>;
    fn get_cursor_offset(&self) -> math::Vec2;
}

#[derive(PartialEq, Debug, Clone)]
pub enum Input {
    Forward,
    Backwards,
    Left,
    Right,
    /// Cursor offset
    MoveCursor(math::Vec2),
    /// y offset
    Scroll(f32),
}
