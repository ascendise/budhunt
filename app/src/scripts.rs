use ace::{component, event, math, vec2, vec3};

#[cfg(test)]
mod tests;

pub struct MovementScript {
    clock: Box<dyn ace::Clock>,
}
impl ace::Script for MovementScript {
    fn run(&self, player: &[&ace::Components], events: &ace::Events) -> Vec<ace::Components> {
        let camera = player
            .iter()
            .find(|c| matches!(c, ace::Components::Position(_)))
            .expect("No position for player entity found");
        let mut camera = component!(camera, ace::Components::Position).clone();
        let inputs = events.get_events(|e| event!(e, ace::Event::Input));
        let cursor_offset = inputs
            .iter()
            .find(|i| matches!(i, ace::Input::MoveCursor(_)))
            .map(|i| component!(i, ace::Input::MoveCursor).clone())
            .unwrap_or(vec2!(0.0));
        let move_dir = self.turn_camera(&mut camera, &cursor_offset);
        let velocity = self.get_camera_velocity(&inputs, &move_dir);
        let mut rigid_body = player
            .iter()
            .find(|c| matches!(c, ace::Components::RigidBody(_)))
            .map(|c| component!(c, ace::Components::RigidBody))
            .expect("no physics for player entity found")
            .clone();
        rigid_body.set_velocity(velocity);
        vec![
            ace::Components::Position(camera),
            ace::Components::RigidBody(rigid_body),
        ]
    }
}
impl MovementScript {
    pub fn new(clock: Box<dyn ace::Clock>) -> Self {
        Self { clock }
    }

    /// Moves camera on xyz-axis and returns movement direction (y axis rotation)
    fn turn_camera(&self, camera: &mut ace::Position, offset: &math::Vec2) -> math::Vec3 {
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

    fn get_camera_velocity(
        &self,
        inputs: &[ace::Input],
        move_direction: &math::Vec3,
    ) -> math::Vec3 {
        let mut movement = math::Vec3::default();
        let speed = 10.0;
        let speed = self.clock.time_delta() * speed;
        let front = move_direction.normalize();
        let up = vec3!(0.0, 1.0, 0.0);
        let strafe = front.cross(&up).normalize();
        if inputs.contains(&ace::Input::Forward) {
            movement = &movement + &front;
        }
        if inputs.contains(&ace::Input::Backwards) {
            movement = &movement - &front;
        }
        if inputs.contains(&ace::Input::Right) {
            movement = &(&movement / 2.0) + &strafe;
        }
        if inputs.contains(&ace::Input::Left) {
            movement = &(&movement / 2.0) - &strafe;
        }
        movement * speed
    }
}
