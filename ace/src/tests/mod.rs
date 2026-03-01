use crate::*;
use pretty_assertions::assert_eq;

mod entities_tests;

#[test]
pub fn component_should_return_specific_component_type() {
    //Arrange
    let component: Component = Component::Position(Default::default());
    //Act
    let position = component!(component, Component::Position);
    //Assert
    assert_eq!(Position::default(), position); // At this point we already asserted that we can get
    // the inner value
}

#[test]
pub fn component_should_return_specific_component_type_for_options() {
    //Arrange
    let component: Component = Component::Position(Default::default());
    let component: Option<Component> = Some(component);
    //Act
    let position = component!(component, Some(Component::Position));
    //Assert
    assert_eq!(Position::default(), position); // At this point we already asserted that we can get
    // the inner value
}

#[test]
#[should_panic]
pub fn component_should_panic_when_getting_wrong_type() {
    //Arrange
    let component: Component = Component::Position(Default::default());
    //Act
    let _ = component!(component, Component::Model);
}
