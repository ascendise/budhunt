use ace_proc_macros::Component;
use std::ops::{Index, IndexMut};

use indexmap::{IndexMap, map::Entry};

pub mod events;
pub use events::Events;
pub mod gfx;
pub mod glfw_input;
pub mod math;
pub mod physics;
pub mod scripts;
pub use scripts::Script;

use crate::physics::CollisionEvent;

#[cfg(test)]
mod tests;

#[macro_export]
/// Used to quickly map a Component enum variant to it's inner value
///
/// # Usage
/// ```
/// use ace::component;
/// // used for implementing custom components
/// use ace_proc_macros::Component;
/// use ace::Component;
///
/// #[derive(Component, PartialEq, Clone, Debug)]
/// enum MyComponents { CompA(usize), CompB(f32), CompC}
///
/// // Map component to known type
/// let comp: MyComponents = MyComponents::CompA(42);
/// let value: usize = component!(comp, MyComponents::CompA);
/// assert_eq!(42, value);
///
/// // Map option to known type
/// let comp = Some(MyComponents::CompA(42));
/// let value: usize = component!(comp, Some(MyComponents::CompA));
/// assert_eq!(42, value);
///
/// // Map option to known type or return default
/// let comp = None;
/// let value: usize = component!(comp, Some(MyComponents::CompA) or 42);
/// assert_eq!(42, value);
/// ```
/// # Panics
/// If you assume the wrong component variant, the macro will panic
macro_rules! component {
    ($v:expr, Some($e:path)) => {
        match $v {
            Some($e(v)) => v,
            _ => panic!(stringify!($e)),
        }
    };
    ($v:expr, Some($e:path) or $default:expr) => {
        match $v {
            Some($e(v)) => v,
            _ => $default,
        }
    };
    ($v:expr, $e:path) => {
        match $v {
            $e(v) => v,
            _ => panic!(stringify!($e)),
        }
    };
}

#[macro_export]
/// Transforms an [Option<Component>] into an [Option<T>]. If the component does not match,
/// [maybe_component] returns [None]
///
/// # Usage
/// ```
/// use ace::maybe_component;
/// // used for implementing custom components
/// use ace_proc_macros::Component;
/// use ace::Component;
///
/// #[derive(Component, PartialEq, Clone, Debug)]
/// enum MyComponents { CompA(usize), CompB(f32), CompC}
///
/// // Map component with matching type
/// let comp: Option<MyComponents> = Some(MyComponents::CompA(42));
/// let value: Option<usize> = maybe_component!(comp, MyComponents::CompA);
/// assert_eq!(Some(42), value);
///
/// // Map component with mismatching type
/// let comp: Option<MyComponents> = Some(MyComponents::CompA(42));
/// let value: Option<f32> = maybe_component!(comp, MyComponents::CompB);
/// assert_eq!(None, value);
/// ```
///
macro_rules! maybe_component {
    ($v:expr, $e:path) => {
        match $v {
            Some($e(v)) => Some(v),
            _ => None,
        }
    };
}

pub struct World {
    entities: Entities,
    systems: Vec<Box<dyn System>>,
    clock: Box<dyn Clock>,
    input_listener: Box<dyn InputListener>,
}
impl World {
    pub fn init(
        entities: Entities,
        systems: Vec<Box<dyn System>>,
        clock: Box<dyn Clock>,
        input_listener: Box<dyn InputListener>,
    ) -> Self {
        Self {
            entities,
            systems,
            clock,
            input_listener,
        }
    }

    pub fn run_frame(&mut self) {
        self.clock.stop_frame_time();
        let events = Events::empty();
        let inputs = self.input_listener.get_inputs();
        for input in inputs {
            events.push_event(Event::Input(input));
        }
        for system in &self.systems {
            system.run(&mut self.entities, &events)
        }
    }
}

#[derive(Debug)]
pub struct Entities<T: Component = Components, const E: usize = 255> {
    components: IndexMap<u32, [Option<T>; E]>,
    empty_bucket: [Option<T>; E],
    entities_count: usize,
    register: [u32; E],
}
impl Entities {
    pub fn empty() -> Self {
        Self::empty_custom()
    }

    pub fn empty_custom<T: Component, const E: usize>() -> Entities<T, E> {
        let components = Default::default();
        let empty_bucket = [0; E].map(|_| None);
        Entities::<T, E> {
            components,
            empty_bucket,
            entities_count: 0,
            register: [0u32; E],
        }
    }
}
impl<T: Component, const E: usize> Entities<T, E> {
    pub fn count(&self) -> usize {
        self.entities_count
    }

    pub fn create_entity(&mut self, entity: Vec<T>) -> usize {
        let entity_id = self.entities_count;
        self.entities_count = entity_id + 1;
        for component in entity {
            self.update_entity(entity_id, component);
        }
        entity_id
    }

    pub fn get_entity(&self, entity_id: usize) -> Vec<&T> {
        self.components
            .iter()
            .flat_map(|(_, b)| &b[entity_id])
            .collect()
    }

    pub fn update_entity(&mut self, entity_id: usize, component: T) {
        let type_id = component.get_type();
        self.register[entity_id] |= type_id;
        if component.is_marker() {
            return;
        }
        let bucket = self.get_or_create_bucket(type_id);
        bucket[entity_id] = Some(component);
    }

    pub fn update_entity_batch(&mut self, entity_id: usize, components: Vec<T>) {
        for component in components {
            self.update_entity(entity_id, component);
        }
    }

    fn get_or_create_bucket(&mut self, type_id: u32) -> &mut [Option<T>] {
        match self.components.entry(type_id) {
            Entry::Occupied(b) => b.into_mut(),
            Entry::Vacant(e) => e.insert([0; E].map(|_| None)),
        }
    }

    pub fn get_components(&self, component_type: u32) -> Vec<&T> {
        self.get_bucket(component_type).iter().flatten().collect()
    }

    /// Returns slice of bucket containing the specified component type.
    /// Length of slice is equal to [Entities::count].
    pub fn get_bucket(&self, component_type: u32) -> &[Option<T>] {
        let bucket = self
            .components
            .get(&component_type)
            .unwrap_or(&self.empty_bucket);
        &bucket[0..self.entities_count]
    }

    /// Takes a set of bitflags OR'd together and returns filtered (only specified components) entities.
    fn get_entities(&self, components: u32) -> Vec<(usize, Vec<&T>)> {
        let mut entities = vec![];
        for e in 0..self.entities_count {
            let entity = self.register[e];
            if entity & components >= components {
                let entity = self.get_entity(e);
                entities.push((e, entity));
            }
        }
        entities
    }
}
impl<T: Component, const E: usize> IndexMut<u32> for Entities<T, E> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.components
            .get_mut(&index)
            .expect("Access to unknown component type")
    }
}

impl<T: Component, const E: usize> Index<u32> for Entities<T, E> {
    type Output = [Option<T>; E];

    fn index(&self, index: u32) -> &Self::Output {
        self.components
            .get(&index)
            .expect("Access to unknown component type")
    }
}

#[derive(Component)]
pub enum Components {
    Position(Position),
    Model(gfx::Model),
    Light(gfx::Light),
    Scripts(Vec<Box<dyn scripts::Script>>),
    Player,
    Collider(physics::Collider),
    RigidBody(physics::RigidBody),
}

pub trait Component {
    /// Returns the bitflag indicating the specific component type.
    fn get_type(&self) -> u32;
    /// Returns if the component is a marker component, e.g. a component
    /// without data, which has no bucket.
    fn is_marker(&self) -> bool;
}

pub trait System {
    fn run(&self, entities: &mut Entities, events: &Events);
}

pub trait Clock {
    /// Returns time since last frame in seconds.
    fn time_delta(&self) -> f32;
    /// Updates time delta.
    fn stop_frame_time(&self);
}

// Split component into Position and Direction as Direction is really only needed for rendering
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Position {
    pub position: math::Vec3,
    pub direction: math::Vec3,
}

pub trait InputListener {
    fn get_inputs(&self) -> Vec<Input>;
    fn get_cursor_offset(&self) -> math::Vec2;
}

#[derive(PartialEq, Debug, Clone)]
pub enum Input {
    Forward,
    Backwards,
    Left,
    Right,
    /// Cursor offset
    MoveCursor(math::Vec2),
    /// y offset
    Scroll(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Input(Input),
    Collision(CollisionEvent),
}
