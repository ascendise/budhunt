use crate::events::Events;
use pretty_assertions::assert_eq;

#[derive(Debug, PartialEq, Eq, Clone)]
enum TestEvent {
    Hello,
    Goodbye,
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
    assert_eq!(*events, vec![TestEvent::Hello, TestEvent::Goodbye]);
    assert_eq!(new_events, vec![]);
}

#[test]
pub fn handle_events_should_pop_events_matching_predicate() {
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
    let matching = sut.handle_events(|e| matches!(e, TestEvent::Hello));
    // Assert
    assert_eq!(
        matching,
        vec![TestEvent::Hello, TestEvent::Hello, TestEvent::Hello]
    );
    let events = sut.events.lock().unwrap();
    assert_eq!(
        *events,
        vec![TestEvent::Goodbye],
        "handled events not removed!"
    );
}
