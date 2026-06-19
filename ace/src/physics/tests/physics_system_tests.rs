use pretty_assertions::assert_eq;
use pretty_assertions::assert_ne;

use crate::math;
use crate::physics::Collider;
use crate::physics::CollisionSystem;
use crate::{
    Components, Events, Position, System, component,
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
        Components::Position(Position::default()),
    ]);
    sut.run(&mut entities, &Events::empty());
    // Assert
    let positions = entities.get_components(Components::POSITION);
    let position = component!(positions.first(), Some(Components::Position));
    assert_ne!(&Position::default(), position, "entity did not move!");
    assert_eq!(vec3!(1.0), position.position, "entity moved the wrong way!");
}

const CUBE: [math::Vec3; 8] = [
    vec3!(-0.5, -0.5, 0.5),
    vec3!(0.5, -0.5, 0.5),
    vec3!(0.5, 0.5, 0.5),
    vec3!(-0.5, 0.5, 0.5),
    vec3!(-0.5, 0.5, -0.5),
    vec3!(0.5, 0.5, -0.5),
    vec3!(-0.5, -0.5, -0.5),
    vec3!(0.5, -0.5, -0.5),
];
pub fn cube(size: f32) -> Collider {
    let mut cube = vec![];
    for vertex in CUBE {
        cube.push(vertex * size);
    }
    Collider::new(cube)
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
        Components::Position(Position::default()),
    ]);
    let moving_entity = entities.create_entity(vec![
        Components::RigidBody(RigidBody::new(vec3!(1.0, 0.0, 0.0))),
        Components::Collider(cube(1.0)),
        Components::Position(Position {
            position: vec3!(-1.1, 0.0, 0.0),
            direction: Default::default(),
        }),
    ]);
    sut.run(&mut entities, &Events::empty());
    // Assert
    let positions = entities.get_components(Components::POSITION);
    assert_eq!(
        vec3!(0.0),
        component!(positions[static_entity], Components::Position).position,
        "static entity was displaced!",
    );
    assert_eq!(
        vec3!(-1.0, 0.0, 0.0),
        component!(positions[moving_entity], Components::Position).position,
        "moving entity at wrong position!"
    );
}
