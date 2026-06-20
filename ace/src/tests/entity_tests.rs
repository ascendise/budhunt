use crate::{Component, Entity};
use pretty_assertions::assert_eq;

#[derive(Component, Debug, PartialEq)]
pub enum Value {
    Integer(u32),
    Bool(bool),
}

#[test]
pub fn indexing_entity_should_return_component_by_type_id() {
    // Arrange
    let integer = Value::Integer(123);
    let boolean = Value::Bool(true);
    let sut = Entity::new(0, vec![&integer, &boolean]);
    // Act
    let value = &sut[Value::INTEGER];
    // Assert
    assert_eq!(&integer, value);
}

#[test]
#[should_panic]
pub fn indexing_entity_should_panic_if_component_is_missing() {
    // Arrange
    let boolean = Value::Bool(true);
    let sut = Entity::new(0, vec![&boolean]);
    // Act
    let _ = &sut[Value::INTEGER];
    // Assert
    unreachable!("a non-existing value was returned without panic!");
}
