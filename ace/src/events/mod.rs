use std::sync::{Arc, Mutex};

use crate::physics::CollisionEvent;

#[cfg(test)]
mod tests;

/// Used to test events and get the inner value
///
/// # Usage
///
/// ```
/// /// Test if an event is a specific variant
/// use ace::event;
/// #[derive(Debug, PartialEq, Eq)]
/// enum MyEvents { A, B, C(usize), D(usize) }
/// let event = MyEvents::A;
/// assert_eq!(Some(MyEvents::A), event!(event, is MyEvents::A));
/// assert_eq!(None, event!(event, is MyEvents::B));
/// /// Test if an event is a specific variant and get it's inner value
/// let event = MyEvents::C(42);
/// assert_eq!(Some(42), event!(event, MyEvents::C));
/// assert_eq!(None, event!(event, MyEvents::D));
/// ```
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
