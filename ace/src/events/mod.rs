use std::sync::{Arc, Mutex};

use crate::physics::CollisionEvent;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! event {
    ($v:expr, $e:path) => {
        match $v {
            $e(v) => Some(v.clone()),
            _ => None,
        }
    };
    ($v:expr, is $e:path) => {
        match $v {
            $e => Some($e),
            _ => None,
        }
    };
}

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
impl<E> Events<E> {
    pub fn push_event(&self, event: E) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn push_events(&self, new_events: &mut Vec<E>) {
        let mut events = self.events.lock().unwrap();
        events.append(new_events);
    }
    pub fn handle_events<F, T>(&self, mut predicate: F) -> Vec<T>
    where
        F: FnMut(&E) -> Option<T>,
    {
        let mut events = self.events.lock().unwrap();
        let matching = events.iter().filter_map(&mut predicate).collect();
        events.retain(|e| predicate(e).is_none());
        matching
    }
}

#[derive(Clone)]
pub enum Event {
    Collision(CollisionEvent),
}
