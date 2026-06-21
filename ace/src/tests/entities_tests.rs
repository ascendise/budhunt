use crate::Component;
use crate::Entities;
use crate::Entity;
use pretty_assertions::assert_eq;

#[derive(Component, PartialEq, Clone, Debug)]
pub enum TestComponents {
    Number(u32),
    Decimal(f32),
    #[allow(dead_code)]
    Bool(bool),
    Marker,
}

#[test]
pub fn create_entity_should_create_new_buckets_for_components() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    // Act
    let component = TestComponents::Number(128);
    let component2 = TestComponents::Decimal(1.2);
    entities.create_entity(vec![component, component2]);
    // Assert
    let bool = entities.components.get(&TestComponents::BOOL);
    assert_eq!(None, bool, "Unexpected text components created");
    let numbers = entities
        .components
        .get(&TestComponents::NUMBER)
        .expect("No numbers created");
    let empty = [0; 32].map(|_| None);
    let mut expected_numbers = empty.clone();
    expected_numbers[0] = Some(TestComponents::Number(128));
    assert_eq!(&expected_numbers, numbers);
    let decimals = entities
        .components
        .get(&TestComponents::DECIMAL)
        .expect("No decimals created");
    let mut expected_decimals = empty.clone();
    expected_decimals[0] = Some(TestComponents::Decimal(1.2));
    assert_eq!(&expected_decimals, decimals);
}

#[test]
pub fn create_entity_should_only_fill_necessary_buckets() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    // Act
    let component = TestComponents::Number(128);
    let component2 = TestComponents::Decimal(1.2);
    entities.create_entity(vec![component.clone()]);
    entities.create_entity(vec![component, component2]);
    // Assert
    let numbers = entities
        .components
        .get(&TestComponents::NUMBER)
        .expect("No numbers created");
    let numbers = &numbers[0..2];
    assert_eq!(
        &[
            Some(TestComponents::Number(128)),
            Some(TestComponents::Number(128))
        ],
        numbers
    );
    let decimals = entities
        .components
        .get(&TestComponents::DECIMAL)
        .expect("No decimals created");
    let decimals = &decimals[0..2];
    assert_eq!(&[None, Some(TestComponents::Decimal(1.2))], decimals);
}

#[test]
pub fn create_entity_should_not_create_bucket_for_markers() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    // Act
    let component = TestComponents::Number(128);
    let marker = TestComponents::Marker;
    entities.create_entity(vec![component, marker]);
    // Assert
    assert_eq!(
        1,
        entities.components.iter().count(),
        "Bucket was created for marker component!"
    );
    assert!(
        !entities
            .get_entities(TestComponents::NUMBER | TestComponents::MARKER)
            .is_empty(),
        "Marker component was not added to register!"
    );
}

#[test]
pub fn get_components_should_return_flattened_bucket() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    entities.create_entity(vec![
        TestComponents::Number(128),
        TestComponents::Decimal(1.0),
    ]);
    entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    entities.create_entity(vec![
        TestComponents::Number(256),
        TestComponents::Decimal(3.0),
    ]);
    // Act
    let components = entities.get_components(TestComponents::NUMBER);
    // Assert
    let expected_components = vec![&TestComponents::Number(128), &TestComponents::Number(256)];
    assert_eq!(expected_components, components);
}

#[test]
pub fn get_components_should_return_empty_bucket_when_no_components_found() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    entities.create_entity(vec![
        TestComponents::Number(128),
        TestComponents::Decimal(1.0),
    ]);
    entities.create_entity(vec![TestComponents::Decimal(3.0)]);
    // Act
    let components = entities.get_components(TestComponents::BOOL);
    // Assert
    let expected_components = Vec::<&TestComponents>::new();
    assert_eq!(expected_components, components);
}

#[test]
pub fn get_bucket_should_return_all_components_up_to_entity_count() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    entities.create_entity(vec![
        TestComponents::Number(128),
        TestComponents::Decimal(1.0),
    ]);
    entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    entities.create_entity(vec![
        TestComponents::Number(256),
        TestComponents::Decimal(3.0),
    ]);
    // Act
    let bucket = entities.get_bucket(TestComponents::NUMBER);
    // Assert
    let expected_bucket = vec![
        Some(TestComponents::Number(128)),
        None,
        Some(TestComponents::Number(256)),
    ];
    assert_eq!(expected_bucket, bucket);
    assert_eq!(entities.count(), bucket.len());
}

#[test]
pub fn get_entity_should_return_all_components_of_one_entity() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    let component1 = TestComponents::Number(128);
    let component2 = TestComponents::Decimal(1.0);
    let entity_id = entities.create_entity(vec![component1.clone(), component2.clone()]);
    // Act
    let entity = entities.get_entity(entity_id);
    // Assert
    let expected_entity = Entity::new(entity_id, vec![&component1, &component2]);
    assert_eq!(expected_entity, entity);
}

#[test]
pub fn get_entities_should_return_all_entities_with_matching_flags() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    entities.create_entity(vec![
        TestComponents::Decimal(1.0),
        TestComponents::Number(128),
    ]); // Target 1 
    entities.create_entity(vec![
        TestComponents::Decimal(2.0),
        TestComponents::Number(256),
        TestComponents::Bool(true),
    ]); // Target 2
    entities.create_entity(vec![
        TestComponents::Bool(false),
        TestComponents::Decimal(1.0),
    ]);
    entities.create_entity(vec![
        TestComponents::Number(128),
        TestComponents::Bool(true),
    ]);
    // Act
    let entities = entities.get_entities(TestComponents::DECIMAL | TestComponents::NUMBER);
    // Assert
    let expected_entities = vec![
        Entity::new(
            0,
            vec![&TestComponents::Decimal(1.0), &TestComponents::Number(128)],
        ),
        Entity::new(
            1,
            vec![
                &TestComponents::Decimal(2.0),
                &TestComponents::Number(256),
                &TestComponents::Bool(true),
            ],
        ),
    ];
    assert_eq!(expected_entities, entities);
}

#[test]
pub fn update_and_commit_should_update_existing_component() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    let entity1 = entities.create_entity(vec![TestComponents::Decimal(1.0)]);
    let entity2 = entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    // Act
    let mut updates = entities.update();
    updates
        .set(entity1, TestComponents::Decimal(11.0))
        .set(entity2, TestComponents::Decimal(12.0));
    entities.commit(updates);
    // Assert
    assert_eq!(
        &Some(TestComponents::Decimal(11.0)),
        &entities[TestComponents::DECIMAL][entity1]
    );
    assert_eq!(
        &Some(TestComponents::Decimal(12.0)),
        &entities[TestComponents::DECIMAL][entity2]
    );
}

#[test]
pub fn update_and_commit_should_add_new_component() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    let entity1 = entities.create_entity(vec![TestComponents::Decimal(1.0)]);
    let entity2 = entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    // Act
    let mut updates = entities.update();
    updates
        .set(entity1, TestComponents::Number(11))
        .set(entity2, TestComponents::Number(12));
    entities.commit(updates);
    // Assert
    let expected_flags = TestComponents::DECIMAL | TestComponents::NUMBER;
    assert_eq!(
        expected_flags, entities.register[entity1],
        "Bitflags were not updated!"
    );
    assert_eq!(
        expected_flags, entities.register[entity2],
        "Bitflags were not updated!"
    );
    assert_eq!(
        &Some(TestComponents::Number(11)),
        &entities[TestComponents::NUMBER][entity1]
    );
    assert_eq!(
        &Some(TestComponents::Number(12)),
        &entities[TestComponents::NUMBER][entity2]
    );
}

#[test]
pub fn update_and_commit_should_handle_marker_component() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    let entity1 = entities.create_entity(vec![TestComponents::Decimal(1.0)]);
    let entity2 = entities.create_entity(vec![TestComponents::Decimal(2.0)]);
    // Act
    let mut updates = entities.update();
    updates
        .set(entity1, TestComponents::Marker)
        .set(entity2, TestComponents::Marker);
    entities.commit(updates);
    // Assert
    let expected_flags = TestComponents::DECIMAL | TestComponents::MARKER;
    assert_eq!(
        expected_flags, entities.register[entity1],
        "Bitflags were not updated!"
    );
    assert_eq!(
        expected_flags, entities.register[entity2],
        "Bitflags were not updated!"
    );
    assert_eq!(
        &None,
        &entities.components.get(&TestComponents::MARKER),
        "Bucket for marker component created!"
    );
}

#[test]
pub fn update_and_commit_should_allow_setting_multiple_components_at_once() {
    // Arrange
    let mut entities = Entities::empty_custom::<TestComponents, 32>();
    let entity = entities.create_entity(vec![TestComponents::Decimal(1.0)]);
    // Act
    let mut updates = entities.update();
    updates.set_batch(
        entity,
        vec![TestComponents::Number(11), TestComponents::Marker],
    );
    entities.commit(updates);
    // Assert
    let expected_flags = TestComponents::NUMBER | TestComponents::DECIMAL | TestComponents::MARKER;
    assert_eq!(
        expected_flags, entities.register[entity],
        "Bitflags were not updated!"
    );
    assert_eq!(
        &Some(TestComponents::Number(11)),
        &entities[TestComponents::NUMBER][entity],
        "Bucket for marker component created!"
    );
}

#[test]
#[should_panic]
pub fn update_and_commit_should_panic_if_pending_updates_drop_before_commit() {
    // Arrange
    let entities = Entities::empty_custom::<TestComponents, 32>();
    let mut updates = entities.update();
    updates.set(0, TestComponents::Number(123));
    // Act
    // No Entities::commit();
    // Assert
    // Panics!
}
