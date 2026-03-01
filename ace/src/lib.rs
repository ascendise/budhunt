use std::ops::{Index, IndexMut};

use indexmap::IndexMap;

use crate::input::InputListener;

pub mod gfx;
pub mod input;
pub mod math;

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
    input_listener: Box<dyn input::InputListener>,
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
        for component in entity {
            let type_id = component.get_type();
            if !self.components.contains_key(&type_id) {
                let empty_bucket = [0; E].map(|_| None);
                self.components.insert(type_id, empty_bucket);
            };
            let bucket = self.components.get_mut(&type_id).unwrap();
            bucket[idx] = Some(component);
        }
        idx
    }

    pub fn get_entity(&self, idx: usize) -> Vec<&T> {
        self.components.iter().flat_map(|(_, b)| &b[idx]).collect()
    }

    pub fn update_entity(&mut self, idx: usize, value: T) {
        let type_id = value.get_type();
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

bitflags::bitflags! {
pub struct Components: u32 {
    const POSITION = 0b100;
    const MODEL = 0b10;
    const LIGHT = 0b1;
}}

pub enum Component {
    Position(Position),
    Model(gfx::Model),
    Light(gfx::Light),
}
impl Component {
    pub const POSITION: u32 = Components::POSITION.0.0;
    pub const MODEL: u32 = Components::MODEL.0.0;
    pub const LIGHT: u32 = Components::LIGHT.0.0;
}
impl TypeId for Component {
    fn get_type(&self) -> u32 {
        match self {
            Component::Position(_) => Component::POSITION,
            Component::Model(_) => Component::MODEL,
            Component::Light(_) => Component::LIGHT,
        }
    }
}

pub trait TypeId {
    fn get_type(&self) -> u32;
}

pub trait System {
    fn run(&self, entities: &mut Entities, inputs: &[input::Input]);
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
