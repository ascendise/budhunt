use std::ops::{Index, IndexMut};

use indexmap::IndexMap;

pub mod gfx;
pub mod glfw_input;
pub mod math;
pub mod scripts;
pub use scripts::Script;

#[cfg(test)]
mod tests;

#[macro_export]
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
        let inputs = self.input_listener.get_inputs();
        for system in &self.systems {
            system.run(&mut self.entities, &inputs)
        }
    }
}

#[derive(Debug)]
pub struct Entities<T: TypeId = Component, const E: usize = 255> {
    components: IndexMap<u32, [Option<T>; E]>,
    empty_bucket: [Option<T>; E],
    entities: usize,
    register: [u32; E],
}
impl Entities {
    pub const PLAYER_IDX: usize = 0;

    pub fn empty() -> Self {
        Self::empty_custom()
    }

    pub fn empty_custom<T: TypeId, const E: usize>() -> Entities<T, E> {
        let components = Default::default();
        let empty_bucket = [0; E].map(|_| None);
        Entities::<T, E> {
            components,
            empty_bucket,
            entities: 0,
            register: [0u32; E],
        }
    }
}
impl<T: TypeId, const E: usize> Entities<T, E> {
    pub fn count(&self) -> usize {
        self.entities
    }

    pub fn add_entity(&mut self, entity: Vec<T>) -> usize {
        let idx = self.entities;
        self.entities = idx + 1;
        let mut components = 0u32;
        for component in entity {
            let type_id = component.get_type();
            components |= type_id;
            if !self.components.contains_key(&type_id) {
                let empty_bucket = [0; E].map(|_| None);
                self.components.insert(type_id, empty_bucket);
            };
            let bucket = self.components.get_mut(&type_id).unwrap();
            bucket[idx] = Some(component);
        }
        self.register[idx] = components;
        idx
    }

    pub fn get_entity(&self, idx: usize) -> Vec<&T> {
        self.components.iter().flat_map(|(_, b)| &b[idx]).collect()
    }

    pub fn update_entity(&mut self, idx: usize, value: T) {
        let type_id = value.get_type();
        if self.register[idx] & value.get_type() == 0 {
            let empty_bucket = [0; E].map(|_| None);
            self.components.insert(value.get_type(), empty_bucket);
        }
        self[type_id][idx] = Some(value)
    }

    pub fn get_components(&self, component_type: u32) -> Vec<&T> {
        self.get_bucket(component_type).iter().flatten().collect()
    }

    /// Returns slice of bucket containing the specified component type
    /// Length of slice is equal to [Entities::count]
    pub fn get_bucket(&self, component_type: u32) -> &[Option<T>] {
        let bucket = self
            .components
            .get(&component_type)
            .unwrap_or(&self.empty_bucket);
        &bucket[0..self.entities]
    }

    /// Takes a set of bitflags OR'd together and returns filtered (only specified components) entities
    fn get_entities(&self, components: u32) -> Vec<(usize, Vec<&T>)> {
        let mut entities = vec![];
        for e in 0..self.entities {
            let entity = self.register[e];
            if entity & components >= components {
                let entity = self.get_entity(e);
                entities.push((e, entity));
            }
        }
        entities
    }
}
impl<T: TypeId, const E: usize> IndexMut<u32> for Entities<T, E> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.components
            .get_mut(&index)
            .expect("Access to unknown component type")
    }
}

impl<T: TypeId, const E: usize> Index<u32> for Entities<T, E> {
    type Output = [Option<T>; E];

    fn index(&self, index: u32) -> &Self::Output {
        self.components
            .get(&index)
            .expect("Access to unknown component type")
    }
}

pub enum Component {
    Position(Position),
    Model(gfx::Model),
    Light(gfx::Light),
    Scripts(Vec<Box<dyn scripts::Script>>),
}
impl Component {
    pub const POSITION: u32 = 0b1;
    pub const MODEL: u32 = 0b10;
    pub const LIGHT: u32 = 0b100;
    pub const SCRIPTS: u32 = 0b1000;
}
impl TypeId for Component {
    fn get_type(&self) -> u32 {
        match self {
            Component::Position(_) => Component::POSITION,
            Component::Model(_) => Component::MODEL,
            Component::Light(_) => Component::LIGHT,
            Component::Scripts(_) => Component::SCRIPTS,
        }
    }
}

pub trait TypeId {
    fn get_type(&self) -> u32;
}

pub trait System {
    fn run(&self, entities: &mut Entities, inputs: &[Input]);
}

pub trait Clock {
    /// Returns time since last frame in seconds
    fn time_delta(&self) -> f32;
    /// Updates time delta
    fn stop_frame_time(&self);
}

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
