use crate::*;
use pretty_assertions::assert_eq;

mod entities_tests;

#[test]
pub fn component_should_return_specific_component_type() {
    //Arrange
    let component: Components = Components::Position(vec3!(1.23));
    //Act
    let position = component!(component, Components::Position);
    //Assert
    assert_eq!(vec3!(1.23), position);
}

#[test]
pub fn component_should_return_specific_component_type_for_options() {
    //Arrange
    let component: Components = Components::Position(vec3!(1.23));
    let component: Option<Components> = Some(component);
    //Act
    let position = component!(component, Some(Components::Position));
    //Assert
    assert_eq!(vec3!(1.23), position);
}

#[test]
#[should_panic]
pub fn component_should_panic_when_getting_wrong_type() {
    //Arrange
    let component: Components = Components::Position(Default::default());
    //Act
    let _ = component!(component, Components::Model);
}
