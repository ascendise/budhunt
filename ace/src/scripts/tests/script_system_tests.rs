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
    entities.create_entity(vec![Components::Position(Default::default())]); // Filler
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
    let add_position_script = Box::new(UpdatePositionScript);
    let mut entities = Entities::empty();
    entities.create_entity(vec![Components::Scripts(vec![add_position_script.clone()])]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let entity = entities.get_entity(0);
    let expected_position = vec3!(10.0);
    assert!(
        {
            let mut passed = false;
            for component in entity {
                if let Components::Position(p) = component {
                    assert_eq!(&expected_position, p);
                    passed = true;
                }
            }
            passed
        },
        "Position component was not added"
    );
}

#[test]
pub fn run_should_update_existing_component_with_returned_entity() {
    // Arrange
    let sut = setup();
    let add_position_script = Box::new(UpdatePositionScript);
    let mut entities = Entities::empty();
    let old_position = Components::Position(vec3!(f32::MAX));
    entities.create_entity(vec![
        Components::Scripts(vec![add_position_script.clone()]),
        old_position,
    ]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let entity = entities.get_entity(0);
    let expected_position = vec3!(10.0);
    assert!(
        {
            let mut passed = false;
            for component in entity {
                if let Components::Position(p) = component {
                    assert_eq!(&expected_position, p);
                    passed = true;
                }
            }
            passed
        },
        "Position component was not added"
    );
}

#[derive(Clone)]
pub struct UpdatePositionScript;
impl Script for UpdatePositionScript {
    fn run(&self, _: &[&Components], _: &Events) -> Vec<Components> {
        vec![Components::Position(vec3!(10.0))]
    }
}
