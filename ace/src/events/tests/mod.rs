use crate::{event, events::Events};
use pretty_assertions::assert_eq;

#[derive(Debug, PartialEq, Eq, Clone)]
enum TestEvent {
    Hello,
    Goodbye,
    Value(usize),
}

#[test]
pub fn push_event_should_add_single_event() {
    // Arrange
    let sut = Events::empty_custom::<TestEvent>();
    // Act
    sut.push_event(TestEvent::Hello);
    sut.push_event(TestEvent::Goodbye);
    // Assert
    let events = sut.events.lock().unwrap();
    assert_eq!(*events, vec![TestEvent::Hello, TestEvent::Goodbye]);
}

#[test]
pub fn push_events_should_add_events_in_bulk() {
    // Arrange
    let sut = Events::empty_custom::<TestEvent>();
    // Act
    let mut new_events = vec![TestEvent::Hello, TestEvent::Goodbye];
    sut.push_events(&mut new_events);
    // Assert
    let events = sut.events.lock().unwrap();
    assert_eq!(vec![TestEvent::Hello, TestEvent::Goodbye], *events);
    assert!(new_events.is_empty());
}

#[test]
pub fn handle_events_should_return_events_matching_predicate() {
    // Arrange
    let sut = Events::empty_custom::<TestEvent>();
    let mut new_events = vec![
        TestEvent::Hello,
        TestEvent::Hello,
        TestEvent::Hello,
        TestEvent::Goodbye,
    ];
    sut.push_events(&mut new_events);
    // Act
    let matching = sut.handle_events(|e| event!(e, is TestEvent::Hello));
    // Assert
    assert_eq!(
        vec![TestEvent::Hello, TestEvent::Hello, TestEvent::Hello],
        matching,
    );
}

#[test]
pub fn handle_events_should_allow_mapping_event_directly_to_inner_value() {
    // Arrange
    let sut = Events::empty_custom::<TestEvent>();
    let mut new_events = vec![
        TestEvent::Hello,
        TestEvent::Value(1),
        TestEvent::Value(2),
        TestEvent::Goodbye,
    ];
    sut.push_events(&mut new_events);
    // Act
    let matching = sut.handle_events(|e| event!(e, TestEvent::Value));
    // Assert
    assert_eq!(vec![1, 2], matching);
}
