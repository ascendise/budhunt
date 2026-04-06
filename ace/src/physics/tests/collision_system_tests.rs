use crate::{
    Components, Entities, Event, Events, Position, System, event,
    physics::{Collider, CollisionEvent, CollisionSystem},
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
    let events = events.handle_events(|e| {
        let event = event!(e, Event::Collision);
        assert!(event.is_some(), "Non-collision event pushed unexpectedly!");
        event
    });
    assert!(!events.is_empty(), "No collision event pushed!");
    let event = events.first().unwrap();
    assert_eq!(&CollisionEvent(0, 1), event);
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
    let events = events.handle_events(|e| event!(e, Event::Collision));
    assert!(events.is_empty(), "Unexpected collision event pushed!");
}
