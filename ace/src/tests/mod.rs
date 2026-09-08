use crate::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

mod entities_tests;
mod entity_tests;

#[derive(Component, PartialEq, Clone, Debug)]
pub enum TestComponents {
    Number(u32),
    Decimal(f32),
    Bool(bool),
    Marker,
}

#[test]
pub fn component_should_return_specific_component_type() {
    //Arrange
    let component: TestComponents = TestComponents::Decimal(1.23);
    //Act
    let decimal = component!(component, TestComponents::Decimal);
    //Assert
    assert_eq!(1.23, decimal);
}

#[test]
#[should_panic]
pub fn component_should_panic_when_getting_wrong_type() {
    //Arrange
    let component: TestComponents = TestComponents::Number(99);
    //Act
    let _ = component!(component, TestComponents::Decimal);
}

#[test]
pub fn component_should_return_specific_component_type_for_options() {
    //Arrange
    let component: TestComponents = TestComponents::Decimal(1.23);
    let component: Option<TestComponents> = Some(component);
    //Act
    let decimal = component!(component, Some(TestComponents::Decimal));
    //Assert
    assert_eq!(1.23, decimal);
}

#[test]
pub fn component_should_return_default_if_optional_component_does_not_exist() {
    //Arrange
    let component: TestComponents = TestComponents::Number(99);
    let component: Option<TestComponents> = Some(component);
    //Act
    let decimal = component!(component, Some(TestComponents::Decimal) or 1.23);
    //Assert
    assert_eq!(1.23, decimal);
}

#[test_case(TestComponents::Decimal(1.23), Some(1.23) ; "option has correct type")]
#[test_case(TestComponents::Number(99), None ; "option does not match expected type")]
pub fn maybe_component_should_filter_option_for_correct_type(
    component: TestComponents,
    expected: Option<f32>,
) {
    // Arrange
    let component = Some(component);
    // Act
    let result = maybe_component!(component, Some(TestComponents::Decimal));
    // Assert
    assert_eq!(expected, result);
}

#[test_case(TestComponents::Decimal(1.23), Some(1.23) ; "option has correct type")]
#[test_case(TestComponents::Number(99), None ; "option does not match expected type")]
pub fn maybe_component_should_filter_component_enum_for_correct_type(
    component: TestComponents,
    expected: Option<f32>,
) {
    // Arrange
    // Act
    let result = maybe_component!(component, TestComponents::Decimal);
    // Assert
    assert_eq!(expected, result);
}
