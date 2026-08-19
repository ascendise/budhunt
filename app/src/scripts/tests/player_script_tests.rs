use ace::gfx::Model;
use ace::scripts::Script;
use ace::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

use crate::scripts::PlayerScript;
use crate::scripts::tests::StubClock;

fn setup(clock: Box<dyn Clock>) -> PlayerScript {
    PlayerScript::new(clock)
}

fn mock_model() -> ace::Components {
    let model = Model {
        nodes: vec![],
        transform: Default::default(),
    };
    ace::Components::Model(model)
}

#[test_case(Input::Forward, vec3!(0.0, 0.0, 1.0))]
#[test_case(Input::Backwards, vec3!(0.0, 0.0, -1.0))]
#[test_case(Input::Left, vec3!(-1.0, 0.0, 0.0))]
#[test_case(Input::Right, vec3!(1.0, 0.0, 0.0))]
pub fn run_should_change_player_velocity_on_matching_input(
    input: Input,
    expected_position: math::Vec3,
) {
    // Arrange
    let clock = Box::new(StubClock { fixed_delta: 0.1 });
    let sut = setup(clock);
    let position = Components::Position(vec3!(0.0));
    let direction = Components::Direction(vec3!(0.0, 0.0, 1.0));
    let rigid_body = Components::RigidBody(Default::default());
    let model = mock_model();
    let camera_entity = Entity::new(0, vec![&position, &direction, &rigid_body, &model]);
    // Act
    let events = Events::empty();
    let move_cursor = Input::MoveCursor(vec2!(90.0, 0.0));
    events.push_event(Event::Input(move_cursor));
    events.push_event(Event::Input(input));
    let updated_components = sut.run(&camera_entity, &events);
    // Assert
    let rigid_body = component!(&updated_components[1], Components::RigidBody);
    assert_float_eq!(Vec3 & expected_position, rigid_body.velocity().unwrap())
}

#[test_case(vec2!(90.0, 0.0), vec3!(0.0, 0.0, 1.0).normalize() ; "look forward")]
#[test_case(vec2!(-90.0, 0.0), vec3!(0.0, 0.0, -1.0).normalize() ; "look backwad")]
#[test_case(vec2!(180.0, 0.0), vec3!(-1.0, 0.0, 0.0) ; "look left")]
#[test_case(vec2!(-180.0, 0.0), vec3!(1.0, 0.0, 0.0) ; "look right")]
pub fn run_should_turn_camera_and_model_on_matching_input(
    cursor_offset: math::Vec2,
    expected_camera_direction: math::Vec3,
) {
    // Arrange
    let clock = Box::new(StubClock { fixed_delta: 0.1 });
    let sut = setup(clock);
    let position = Components::Position(vec3!(0.0));
    let direction = Components::Direction(vec3!(0.0, 0.0, 1.0));
    let rigid_body = Components::RigidBody(Default::default());
    let model = mock_model();
    let camera_entity = Entity::new(0, vec![&position, &direction, &rigid_body, &model]);
    // Act
    let move_cursor = Input::MoveCursor(cursor_offset);
    let events = Events::empty();
    events.push_event(Event::Input(move_cursor));
    let updated_components = sut.run(&camera_entity, &events);
    // Assert
    let camera_direction = component!(&updated_components[0], Components::Direction);
    assert_float_eq!(Vec3 & expected_camera_direction, camera_direction);
    let model = component!(&updated_components[2], Components::Model);
    assert_float_eq!(
        Matrix4 model.transform.rotation,
        math::rotation_fpv(camera_direction)
    );
}
