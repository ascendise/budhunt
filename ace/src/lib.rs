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
            _ => panic!("this is not a {}", stringify!($e)),
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
            _ => panic!("this is not a {}", stringify!($e)),
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
        let mut updates = self.update();
        updates.set_batch(entity_id, entity);
        self.commit(updates);

        entity_id
    }

    pub fn get_entity(&self, entity_id: usize) -> Entity<'_, T> {
        let components = self
            .components
            .iter()
            .flat_map(|(_, b)| &b[entity_id])
            .collect();
        Entity::new(entity_id, components)
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
    pub fn get_entities(&self, components: u32) -> Vec<Entity<'_, T>> {
        let mut entities = vec![];
        for e in 0..self.entities_count {
            let entity = self.register[e];
            if entity & components >= components {
                let entity = self.get_entity(e);
                entities.push(entity);
            }
        }
        entities
    }

    /// Creates an empty [Update] object for staging updated components
    /// # Usage
    /// ```
    /// use ace::{Components, Entities, component, vec3};
    /// let mut entities = Entities::empty();
    /// let id = entities.create_entity(vec![Components::Position(vec3!(0.0))]);
    /// let mut updates = entities.update();
    /// updates.set(id, Components::Position(vec3!(1.0, 2.0, 3.0)));
    /// entities.commit(updates); // Failing to commit causes panic
    /// assert_eq!(
    ///     &vec3!(1.0, 2.0, 3.0),
    ///     component!(&entities[Components::POSITION][id], Some(Components::Position)));
    /// ```
    pub fn update(&self) -> Update<T> {
        Update::new()
    }

    pub fn commit(&mut self, mut updates: Update<T>) {
        for (type_id, mut updates) in updates.updates.drain(0..) {
            for (entity_id, value) in updates.drain(0..) {
                self.register[entity_id] |= type_id;
                if value.is_marker() {
                    continue;
                }
                let bucket = self.get_or_create_bucket(type_id);
                bucket[entity_id] = Some(value);
            }
        }
    }
    fn get_or_create_bucket(&mut self, type_id: u32) -> &mut [Option<T>] {
        match self.components.entry(type_id) {
            Entry::Occupied(b) => b.into_mut(),
            Entry::Vacant(e) => e.insert([0; E].map(|_| None)),
        }
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
/// Set of [components](Component) representing a (part of a) single [Entity] from [Entities]
#[derive(Debug, PartialEq, Clone)]
pub struct Entity<'a, T: Component> {
    id: usize,
    components: IndexMap<u32, &'a T>,
}
impl<'a, T: Component> Entity<'a, T> {
    pub fn new(id: usize, components: Vec<&'a T>) -> Self {
        let mut component_map: IndexMap<u32, &'a T> = IndexMap::new();
        for component in components {
            component_map.insert(component.get_type(), component);
        }
        Self {
            id,
            components: component_map,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
impl<'a, T: Component> Index<u32> for Entity<'a, T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.components
            .get(&index)
            .expect("Tried to access a component missing in this entity")
    }
}
#[derive(Default)]
pub struct Update<T: Component> {
    updates: IndexMap<u32, IndexMap<usize, T>>,
}
impl<T: Component> Update<T> {
    pub fn new() -> Self {
        Self {
            updates: IndexMap::new(),
        }
    }

    pub fn set(&mut self, entity_id: usize, component: T) -> &mut Self {
        let type_id = component.get_type();
        match self.updates.entry(type_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(entity_id, component);
            }
            Entry::Vacant(entry) => {
                entry.insert(indexmap::indexmap! {
                    entity_id => component
                });
            }
        };
        self
    }

    pub fn set_batch(&mut self, entity_id: usize, components: Vec<T>) -> &mut Self {
        for component in components {
            self.set(entity_id, component);
        }
        self
    }
}
impl<T: Component> Drop for Update<T> {
    fn drop(&mut self) {
        if !self.updates.is_empty() {
            panic!("Updates with pending changes dropped! Did you forget Entities::commit()?")
        }
    }
}

#[derive(Component)]
pub enum Components {
    Position(math::Vec3),
    Direction(math::Vec3),
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

pub trait InputListener {
    fn get_inputs(&self) -> Vec<Input>;
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
