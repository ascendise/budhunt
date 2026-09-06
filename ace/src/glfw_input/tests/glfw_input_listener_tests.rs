use crate::{
    Input, InputListener,
    glfw_input::{GlfwInputListener, tests::FakeGlfwInputs},
};
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test_case(glfw::Key::W, Input::Forward)]
#[test_case(glfw::Key::S, Input::Backwards)]
#[test_case(glfw::Key::D, Input::Right)]
#[test_case(glfw::Key::A, Input::Left)]
pub fn get_inputs_should_return_move_input_on_movement_key_input(
    key: glfw::Key,
    expected_input: Input,
) {
    // Arrange
    let fake_inputs = FakeGlfwInputs::new(vec![key]);
    let sut = GlfwInputListener::init(Box::new(fake_inputs));
    // Act
    let inputs = sut.get_inputs();
    // Assert
    let expected_input = vec![expected_input];
    assert_eq!(expected_input, inputs);
}
