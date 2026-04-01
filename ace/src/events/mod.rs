use std::sync::{Arc, Mutex};

use crate::physics::CollisionEvent;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Events<T = Event> {
    events: Arc<Mutex<Vec<T>>>,
}
impl Events {
    pub fn empty() -> Self {
        Self::empty_custom()
    }

    pub fn empty_custom<T>() -> Events<T> {
        let events = Arc::new(Mutex::new(vec![]));
        Events::<T> { events }
    }
}
impl<T> Events<T> {
    pub fn push_event(&self, event: T) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn push_events(&self, new_events: &mut Vec<T>) {
        let mut events = self.events.lock().unwrap();
        events.append(new_events);
    }
}

pub enum Event {
    Collision(CollisionEvent),
}
