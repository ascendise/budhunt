use std::sync::{Arc, Mutex};

use crate::physics::CollisionEvent;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Events<E = Event> {
    events: Arc<Mutex<Vec<E>>>,
}
impl Events {
    pub fn empty() -> Self {
        Self::empty_custom()
    }

    pub fn empty_custom<E>() -> Events<E> {
        let events = Arc::new(Mutex::new(vec![]));
        Events::<E> { events }
    }
}
impl<E: Clone> Events<E> {
    pub fn push_event(&self, event: E) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn push_events(&self, new_events: &mut Vec<E>) {
        let mut events = self.events.lock().unwrap();
        events.append(new_events);
    }
    pub fn handle_events<F>(&self, mut predicate: F) -> Vec<E>
    where
        F: FnMut(&&E) -> bool,
    {
        let mut events = self.events.lock().unwrap();
        let matching: Vec<E> = events
            .iter()
            .filter_map(|e| if predicate(&e) { Some(e.clone()) } else { None })
            .collect();
        events.retain(|e| !predicate(&e));
        matching
    }
}

#[derive(Clone)]
pub enum Event {
    Collision(CollisionEvent),
}
