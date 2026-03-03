use ace::scripts::Script;
use ace::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

use crate::scripts::MovementScript;
use crate::scripts::tests::StubClock;

fn setup(clock: Box<dyn Clock>) -> MovementScript {
    MovementScript::new(clock)
}

#[test_case(Input::Forward, vec3!(0.0, 0.0, 1.0))]
#[test_case(Input::Backwards, vec3!(0.0, 0.0, -1.0))]
#[test_case(Input::Left, vec3!(-1.0, 0.0, 0.0))]
#[test_case(Input::Right, vec3!(1.0, 0.0, 0.0))]
pub fn run_should_move_player_on_matching_input(input: Input, expected_position: math::Vec3) {
    // Arrange
    let clock = Box::new(StubClock { fixed_delta: 0.1 });
    let sut = setup(clock);
    let camera = Position {
        position: vec3!(0.0),
        direction: vec3!(0.0, 0.0, 1.0),
    };
    let camera = Component::Position(camera);
    let entity = vec![&camera];
    // Act
    let updated_components = sut.run(&entity, &[input, Input::MoveCursor(vec2!(90.0, 0.0))]);
    // Assert
    let camera = component!(&updated_components[0], Component::Position);
    assert_float_eq!(Vec3 expected_position, camera.position)
}

#[test_case(vec2!(90.0, 0.0), vec3!(0.0, 0.0, 1.0).normalize() ; "look forward")]
#[test_case(vec2!(-90.0, 0.0), vec3!(0.0, 0.0, -1.0).normalize() ; "look backwad")]
#[test_case(vec2!(180.0, 0.0), vec3!(-1.0, 0.0, 0.0) ; "look left")]
#[test_case(vec2!(-180.0, 0.0), vec3!(1.0, 0.0, 0.0) ; "look right")]
pub fn run_should_turn_camera_on_matching_input(
    cursor_offset: math::Vec2,
    expected_camera_direction: math::Vec3,
) {
    // Arrange
    let clock = Box::new(StubClock { fixed_delta: 0.1 });
    let sut = setup(clock);
    let camera = Position {
        position: vec3!(0.0),
        direction: vec3!(0.0, 0.0, 1.0),
    };
    let camera = Component::Position(camera);
    let entity = vec![&camera];
    // Act
    let updated_components = sut.run(&entity, &[Input::MoveCursor(cursor_offset)]);
    // Assert
    let camera = component!(&updated_components[0], Component::Position);
    assert_float_eq!(Vec3 expected_camera_direction, camera.direction);
}
