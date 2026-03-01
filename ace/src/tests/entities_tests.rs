use crate::{Entities, TypeId};
use pretty_assertions::assert_eq;

bitflags::bitflags! {
    pub struct ComponentIds: u32 {
        const NUMBER = 0b100;
        const DECIMAL = 0b10;
        const BOOL = 0b1;
    }
}
#[derive(PartialEq, Clone, Debug)]
pub enum Components {
    Number(u32),
    Decimal(f32),
    #[allow(dead_code)]
    Bool(bool),
}
impl Components {
    pub const NUMBER: u32 = ComponentIds::NUMBER.0.0;
    pub const DECIMAL: u32 = ComponentIds::DECIMAL.0.0;
    pub const BOOL: u32 = ComponentIds::BOOL.0.0;
}
impl TypeId for Components {
    fn get_type(&self) -> u32 {
        match self {
            Components::Bool(_) => Self::BOOL,
            Components::Number(_) => Self::NUMBER,
            Components::Decimal(_) => Self::DECIMAL,
        }
    }
}

#[test]
pub fn add_entity_should_create_new_buckets_for_components() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    // Act
    let component = Components::Number(128);
    let component2 = Components::Decimal(1.2);
    entities.add_entity(vec![component, component2]);
    // Assert
    let bool = entities.components.get(&Components::BOOL);
    assert_eq!(None, bool, "Unexpected text components created");
    let numbers = entities
        .components
        .get(&Components::NUMBER)
        .expect("No numbers created");
    let empty = [0; 32].map(|_| None);
    let mut expected_numbers = empty.clone();
    expected_numbers[0] = Some(Components::Number(128));
    assert_eq!(&expected_numbers, numbers);
    let decimals = entities
        .components
        .get(&Components::DECIMAL)
        .expect("No decimals created");
    let mut expected_decimals = empty.clone();
    expected_decimals[0] = Some(Components::Decimal(1.2));
    assert_eq!(&expected_decimals, decimals);
}

#[test]
pub fn add_entity_should_only_fill_necessary_buckets() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    // Act
    let component = Components::Number(128);
    let component2 = Components::Decimal(1.2);
    entities.add_entity(vec![component.clone()]);
    entities.add_entity(vec![component, component2]);
    // Assert
    let numbers = entities
        .components
        .get(&Components::NUMBER)
        .expect("No numbers created");
    let numbers = &numbers[0..2];
    assert_eq!(
        &[Some(Components::Number(128)), Some(Components::Number(128))],
        numbers
    );
    let decimals = entities
        .components
        .get(&Components::DECIMAL)
        .expect("No decimals created");
    let decimals = &decimals[0..2];
    assert_eq!(&[None, Some(Components::Decimal(1.2))], decimals);
}

#[test]
pub fn get_components_should_return_flattened_bucket() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    entities.add_entity(vec![Components::Number(128), Components::Decimal(1.0)]);
    entities.add_entity(vec![Components::Decimal(2.0)]);
    entities.add_entity(vec![Components::Number(256), Components::Decimal(3.0)]);
    // Act
    let components = entities.get_components(Components::NUMBER);
    // Assert
    let expected_components = vec![&Components::Number(128), &Components::Number(256)];
    assert_eq!(expected_components, components);
}

#[test]
pub fn get_components_should_return_empty_bucket_when_no_components_found() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    entities.add_entity(vec![Components::Number(128), Components::Decimal(1.0)]);
    entities.add_entity(vec![Components::Decimal(3.0)]);
    // Act
    let components = entities.get_components(Components::BOOL);
    // Assert
    let expected_components = Vec::<&Components>::new();
    assert_eq!(expected_components, components);
}

#[test]
pub fn get_bucket_should_return_all_components_up_to_entity_count() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    entities.add_entity(vec![Components::Number(128), Components::Decimal(1.0)]);
    entities.add_entity(vec![Components::Decimal(2.0)]);
    entities.add_entity(vec![Components::Number(256), Components::Decimal(3.0)]);
    // Act
    let bucket = entities.get_bucket(Components::NUMBER);
    // Assert
    let expected_bucket = vec![
        Some(Components::Number(128)),
        None,
        Some(Components::Number(256)),
    ];
    assert_eq!(expected_bucket, bucket);
    assert_eq!(entities.count(), bucket.len());
}

#[test]
pub fn get_entity_should_return_all_components_of_one_entity() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    entities.add_entity(vec![Components::Decimal(2.0)]);
    let entity = entities.add_entity(vec![Components::Number(128), Components::Decimal(1.0)]);
    // Act
    let entity = entities.get_entity(entity);
    // Assert
    let expected_entity = vec![&Components::Decimal(1.0), &Components::Number(128)];
    assert_eq!(expected_entity, entity);
}

#[test]
pub fn update_entity_should_update_component() {
    // Arrange
    let mut entities = Entities::empty_custom::<Components, 32>();
    let entity = entities.add_entity(vec![Components::Number(0), Components::Decimal(1.0)]);
    // Act
    entities.update_entity(entity, Components::Number(128));
    // Assert
    let component = &entities[Components::NUMBER][entity];
    assert_eq!(&Some(Components::Number(128)), component);
}
