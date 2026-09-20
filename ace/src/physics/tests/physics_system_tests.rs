use pretty_assertions::assert_eq;
use pretty_assertions::assert_ne;

use crate::physics::CollisionSystem;
use crate::physics::tests::cube;
use crate::x3d::Transform;
use crate::{
    Components, Events, System, component,
    physics::{PhysicsSystem, RigidBody},
    vec3,
};

fn setup() -> PhysicsSystem {
    PhysicsSystem::new(None)
}

fn setup_with_collision() -> PhysicsSystem {
    PhysicsSystem::new(Some(CollisionSystem))
}

#[test]
pub fn run_should_move_entity_along_velocity() {
    // Arrange
    let sut = setup();
    // Act
    let mut entities = crate::Entities::empty();
    entities.create_entity(vec![
        Components::RigidBody(RigidBody::new(vec3!(1.0))),
        Transform::new(vec3!(0.0)).into(),
    ]);
    sut.run(&mut entities, &Events::empty());
    // Assert
    let transforms = entities.get_components(Components::TRANSFORM);
    let transform = component!(transforms.first(), Some(Components::Transform));
    let position = transform.position;
    assert_ne!(vec3!(0.0), position, "entity did not move!");
    assert_eq!(vec3!(1.0), position, "entity moved the wrong way!");
}

#[test]
pub fn run_should_move_colliding_entities_when_using_collision_system() {
    // Arrange
    let sut = setup_with_collision();
    // Act
    let mut entities = crate::Entities::empty();
    let static_entity = entities.create_entity(vec![
        Components::RigidBody(RigidBody::static_body()),
        Components::Collider(cube(1.0)),
        Transform::new(vec3!(0.0)).into(),
    ]);
    let moving_entity = entities.create_entity(vec![
        Components::RigidBody(RigidBody::new(vec3!(1.0, 0.0, 0.0))),
        Components::Collider(cube(1.0)),
        Transform::new(vec3!(-1.1, 0.0, 0.0)).into(),
    ]);
    sut.run(&mut entities, &Events::empty());
    // Assert
    let transforms = entities.get_components(Components::TRANSFORM);
    let static_entity_position =
        component!(&transforms[static_entity], Components::Transform).position;
    let moving_entity_position =
        component!(&transforms[moving_entity], Components::Transform).position;
    assert_eq!(
        vec3!(0.0),
        static_entity_position,
        "static entity was displaced!",
    );
    assert_eq!(
        vec3!(-1.0, 0.0, 0.0),
        moving_entity_position,
        "moving entity at wrong position!"
    );
}
