use crate::input::tests::StubClock;
use crate::math::tests::assert_vec3_eq;
use crate::*;
use crate::{
    input::{Input, InputSystem},
    math,
};
use test_case::test_case;

pub fn setup(clock: Box<dyn Clock>) -> InputSystem {
    InputSystem { clock }
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
    let mut entities = Entities::empty();
    let entity = entities.add_entity(vec![camera]);
    // Act
    sut.run(&mut entities, &[input, Input::MoveCursor(vec2!(90.0, 0.0))]);
    // Assert
    let camera = &entities.get_components(Component::POSITION)[entity];
    let camera = component!(camera, Component::Position);
    assert_vec3_eq(&expected_position, &camera.position)
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
    let mut entities = Entities::empty();
    let entity = entities.add_entity(vec![camera]);
    // Act
    sut.run(&mut entities, &[Input::MoveCursor(cursor_offset)]);
    // Assert
    let camera = &entities.get_components(Component::POSITION)[entity];
    let camera = component!(camera, Component::Position);
    assert_vec3_eq(&expected_camera_direction, &camera.direction)
}
