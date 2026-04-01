use crate::events::Events;
use pretty_assertions::assert_eq;

#[derive(Debug, PartialEq, Eq)]
enum TestEvent {
    Hello,
    Goodbye,
}

#[test]
pub fn push_events_should_add_event() {
    // Arrange
    let sut = Events::empty::<TestEvent>();
    // Act
    sut.push_event(TestEvent::Hello);
    sut.push_event(TestEvent::Goodbye);
    // Assert
    let events = sut.events.lock().unwrap();
    assert_eq!(*events, vec![TestEvent::Hello, TestEvent::Goodbye]);
}
