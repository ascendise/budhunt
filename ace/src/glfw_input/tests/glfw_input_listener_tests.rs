use crate::{
    Input, InputListener,
    glfw_input::{GlfwInputListener, tests::FakeGlfwInputs},
    vec2,
};
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test]
pub fn get_inputs_should_return_default_cursor_offset() {
    // Arrange
    let fake_inputs = FakeGlfwInputs::new();
    let sut = GlfwInputListener::init(Box::new(fake_inputs));
    // Act
    let inputs = sut.get_inputs();
    // Assert
    let expected_input = vec![Input::MoveCursor(vec2!(0.0))];
    assert_eq!(expected_input, inputs);
}

#[test_case(glfw::Key::W, Input::Forward)]
#[test_case(glfw::Key::S, Input::Backwards)]
#[test_case(glfw::Key::D, Input::Right)]
#[test_case(glfw::Key::A, Input::Left)]
pub fn get_inputs_should_return_move_input_on_movement_key_input(
    key: glfw::Key,
    expected_input: Input,
) {
    // Arrange
    let fake_inputs = FakeGlfwInputs::new();
    fake_inputs.key_actions(vec![key]);
    let sut = GlfwInputListener::init(Box::new(fake_inputs));
    // Act
    let inputs = sut.get_inputs();
    // Assert
    let expected_input = vec![expected_input, Input::MoveCursor(vec2!(0.0))];
    assert_eq!(expected_input, inputs);
}

#[test]
pub fn get_inputs_should_move_cursor_offset_based_on_mouse_movement() {
    // Arrange
    let fake_inputs = FakeGlfwInputs::new();
    fake_inputs.cursor_actions(vec![
        vec2!(50.0, 50.0), // 0, 0
        vec2!(60.0, 60.0), // 1, -1
        vec2!(40.0, 40.0), // -1, 1
    ]);
    let sut = GlfwInputListener::init(Box::new(fake_inputs));
    // Act
    let inputs = sut.get_inputs();
    // Assert
    let expected_input = vec![Input::MoveCursor(vec2!(-1.0, 1.0))];
    assert_eq!(expected_input, inputs);
}
