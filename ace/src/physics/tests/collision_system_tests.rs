use crate::{
    Components, Entities, Event, Events, Position, System, event, math,
    physics::{self, Collider, CollisionEvent, CollisionSystem},
    vec3,
};
use pretty_assertions::assert_eq;

#[test]
pub fn run_should_push_event_for_collision() {
    // Arrange
    let sut = CollisionSystem;
    let plane = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane = Collider::new(plane);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Collider(plane.clone()),
        Components::Position(Position {
            position: vec3!(0.0),
            direction: Default::default(),
        }),
    ]);
    entities.create_entity(vec![
        Components::Collider(plane.clone()),
        Components::Position(Position {
            position: vec3!(0.0),
            direction: Default::default(),
        }),
    ]);
    // Act
    let events = Events::empty();
    sut.run(&mut entities, &events);
    // Assert
    let events = events.get_events(|e| {
        let event = event!(e, Event::Collision);
        assert!(event.is_some(), "Non-collision event pushed unexpectedly!");
        event
    });
    assert!(!events.is_empty(), "No collision event pushed!");
    assert_eq!(
        vec![
            CollisionEvent {
                entity_id: 0,
                collision_point: None
            },
            CollisionEvent {
                entity_id: 1,
                collision_point: None
            },
        ],
        events
    );
}

#[test]
pub fn run_should_not_push_event_if_no_collision() {
    // Arrange
    let sut = CollisionSystem;
    let plane = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane = Collider::new(plane);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Collider(plane.clone()),
        Components::Position(Position {
            position: vec3!(0.0),
            direction: Default::default(),
        }),
    ]);
    entities.create_entity(vec![
        Components::Collider(plane.clone()),
        Components::Position(Position {
            position: vec3!(2.0),
            direction: Default::default(),
        }),
    ]);
    // Act
    let events = Events::empty();
    sut.run(&mut entities, &events);
    // Assert
    let events = events.get_events(|e| event!(e, Event::Collision));
    assert!(events.is_empty(), "Unexpected collision event pushed!");
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
pub fn run_should_calculate_collision_point_for_cube_moving_into_static_cube() {
    // Arrange
    let sut = CollisionSystem;
    let mut entities = Entities::empty();
    let static_cube = cube(1.0);
    let static_cube = entities.create_entity(vec![
        Components::Collider(static_cube),
        Components::RigidBody(physics::RigidBody::static_body()),
        Components::Position(Position {
            position: vec3!(0.0),
            direction: Default::default(),
        }),
    ]);
    let moving_cube = cube(1.0);
    let moving_cube = entities.create_entity(vec![
        Components::Collider(moving_cube),
        Components::RigidBody(physics::RigidBody::new(vec3!(1.0, 0.0, 0.0))),
        Components::Position(Position {
            position: vec3!(-0.1, 0.0, 0.0),
            direction: Default::default(),
        }),
    ]);
    // Act
    let events = Events::empty();
    sut.run(&mut entities, &events);
    // Assert
    let expected_events = vec![
        Event::Collision(CollisionEvent {
            entity_id: static_cube,
            collision_point: None, // Static cube should not get displaced
        }),
        Event::Collision(CollisionEvent {
            entity_id: moving_cube,
            collision_point: Some(vec3!(-1.0, 0.0, 0.0)),
        }),
    ];
    assert_eq!(expected_events, events.get_all_events())
}
