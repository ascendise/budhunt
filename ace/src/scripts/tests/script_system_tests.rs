use crate::scripts::{tests::*, *};
use pretty_assertions::assert_eq;

fn setup() -> ScriptSystem {
    ScriptSystem
}

#[test]
pub fn run_should_run_scripts_on_entity() {
    // Arrange
    let sut = setup();
    let spy_script = Box::new(SpyScript::new());
    let mut entities = Entities::empty();
    let scripts = Components::Scripts(vec![
        spy_script.clone(),
        spy_script.clone(),
        spy_script.clone(),
    ]);
    entities.create_entity(vec![scripts]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    assert_eq!(3, *spy_script.run_count.lock().unwrap());
}

#[test]
pub fn run_should_run_script_on_all_entities() {
    // Arrange
    let sut = setup();
    let spy_script = Box::new(SpyScript::new());
    let mut entities = Entities::empty();
    entities.create_entity(vec![Components::Scripts(vec![spy_script.clone()])]);
    entities.create_entity(vec![Components::Scripts(vec![spy_script.clone()])]);
    entities.create_entity(vec![Components::Transform(Default::default())]); // Filler
    entities.create_entity(vec![Components::Scripts(vec![spy_script.clone()])]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    assert_eq!(3, *spy_script.run_count.lock().unwrap());
}

#[test]
pub fn run_should_update_entity_with_returned_entity() {
    // Arrange
    let sut = setup();
    let update_position_script = Box::new(UpdatePositionScript);
    let mut entities = Entities::empty();
    entities.create_entity(vec![Components::Scripts(vec![
        update_position_script.clone(),
    ])]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let entity = entities.get_entity(0);
    assert_eq!(
        vec3!(10.0),
        component!(&entity[Components::TRANSFORM], Components::Transform).position,
        "New component was not added!"
    );
}

#[test]
pub fn run_should_update_existing_component_with_returned_entity() {
    // Arrange
    let sut = setup();
    let update_position_script = Box::new(UpdatePositionScript);
    let mut entities = Entities::empty();
    let old_position = Transform::new(vec3!(f32::MAX));
    entities.create_entity(vec![
        Components::Scripts(vec![update_position_script.clone()]),
        old_position.into(),
    ]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let entity = entities.get_entity(0);
    assert_eq!(
        vec3!(10.0),
        component!(&entity[Components::TRANSFORM], Components::Transform).position,
        "Existing component was not updated!"
    );
}

#[derive(Clone)]
pub struct UpdatePositionScript;
impl Script for UpdatePositionScript {
    fn run(&self, entity: &Entity<'_, Components>, _: &Events, updates: &mut Update<Components>) {
        updates.set(entity.id(), Transform::new(vec3!(10.0)).into());
    }
}

#[test]
pub fn run_should_commit_new_entity_spawned_by_script() {
    // Arrange
    let sut = setup();
    let spawn_entity_script = Box::new(SpawnPositionScript(vec3!(10.0)));
    let mut entities = Entities::empty();
    let _script_entity =
        entities.create_entity(vec![Components::Scripts(vec![spawn_entity_script.clone()])]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    assert_eq!(2, entities.count(), "new entity was not commited!");
    let new_entity = entities.get_entity(1);
    assert_eq!(
        vec3!(10.0),
        component!(&new_entity[Components::TRANSFORM], Components::Transform).position,
        "Existing component was not updated!"
    );
}

#[derive(Clone)]
pub struct SpawnPositionScript(math::Vec3);
impl Script for SpawnPositionScript {
    fn run(&self, _: &Entity<'_, Components>, _: &Events, updates: &mut Update<Components>) {
        updates.spawn(vec![Transform::new(self.0).into()]);
    }
}
