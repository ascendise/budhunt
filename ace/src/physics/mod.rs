use core::panic;

use crate::{
    Component, Components, Entities, Entity, Event, Events, System, component, event, math,
    maybe_component, vec3,
};

#[cfg(test)]
mod tests;

pub struct PhysicsSystem {
    collision_system: Option<CollisionSystem>,
}
impl System for PhysicsSystem {
    fn run(&self, entities: &mut Entities, events: &Events) {
        let mut updates = entities.update();
        for entity in entities.get_entities(Components::RIGIDBODY | Components::POSITION) {
            Self::move_entity(entity, &mut updates);
        }
        entities.commit(updates);
        if let Some(collision_system) = &self.collision_system {
            Self::handle_collisions(entities, events, collision_system);
        }
    }
}

impl PhysicsSystem {
    pub fn new(collision_system: Option<CollisionSystem>) -> Self {
        Self { collision_system }
    }
    fn move_entity(entity: Entity<'_, Components>, updates: &mut crate::Update<Components>) {
        let rigid_body = component!(&entity[Components::RIGIDBODY], Components::RigidBody);
        if let Some(velocity) = &rigid_body.velocity {
            let position = component!(&entity[Components::POSITION], Components::Position);
            let new_position = position + velocity;
            updates.set(entity.id(), Components::Position(new_position));
        }
    }

    fn handle_collisions(
        entities: &mut Entities,
        events: &Events,
        collision_system: &CollisionSystem,
    ) {
        collision_system.run(entities, events);
        let mut updates = entities.update();
        let events = events
            .get_events(|e| event!(e, Event::Collision))
            .into_iter()
            .flat_map(|e| e.collisions);
        for event in events {
            if let Some(collision_point) = event.collision_point {
                updates.set(event.entity_id, Components::Position(collision_point));
            }
        }
        entities.commit(updates);
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RigidBody {
    velocity: Option<math::Vec3>,
}
impl RigidBody {
    pub fn new(velocity: math::Vec3) -> Self {
        Self {
            velocity: Some(velocity),
        }
    }

    pub fn static_body() -> Self {
        Self { velocity: None }
    }

    pub fn set_velocity(&mut self, velocity: math::Vec3) {
        self.velocity = Some(velocity);
    }

    /// Defines current velocity of physical body.
    /// If this value is [None], the body is static (not influenced by external physical forces)
    pub fn velocity(&self) -> Option<&math::Vec3> {
        self.velocity.as_ref()
    }
}

pub struct CollisionSystem;
impl System for CollisionSystem {
    fn run(&self, entities: &mut crate::Entities, events: &Events) {
        let colliders = entities.get_entities(Components::COLLIDER | Components::POSITION); //TODO: do we REQUIRE a position?
        let rigid_bodies = entities.get_bucket(Components::RIGIDBODY);
        for collider in &colliders {
            let mut collision_entity = CollisionEntity {
                collider: component!(&collider[Components::COLLIDER], Components::Collider).clone(),
                position: component!(&collider[Components::POSITION], Components::Position).clone(),
                physics: maybe_component!(
                    &rigid_bodies[collider.id()],
                    Some(Components::RigidBody)
                )
                .cloned(),
            };
            for other in colliders.iter().skip(collider.id() + 1) {
                let mut other_collision_entity = CollisionEntity {
                    collider: component!(&other[Components::COLLIDER], Components::Collider)
                        .clone(),
                    position: component!(&other[Components::POSITION], Components::Position)
                        .clone(),
                    physics: maybe_component!(
                        &rigid_bodies[other.id()],
                        Some(Components::RigidBody)
                    )
                    .cloned(),
                };
                if collision_entity.intersects(&other_collision_entity) {
                    let collider_collision_point =
                        Self::find_collision_point(&mut collision_entity, &other_collision_entity);
                    let obstacle_collision_point =
                        Self::find_collision_point(&mut other_collision_entity, &collision_entity);

                    let event = vec![
                        CollisionEvent {
                            entity_id: collider.id(),
                            collision_point: collider_collision_point,
                        },
                        CollisionEvent {
                            entity_id: other.id(),
                            collision_point: obstacle_collision_point,
                        },
                    ];
                    let event = CompoundCollisionEvent::new(event);
                    events.push_event(Event::Collision(event));
                }
            }
        }
    }
}
impl CollisionSystem {
    fn find_collision_point(
        collider: &mut CollisionEntity,
        obstacle: &CollisionEntity,
    ) -> Option<math::Vec3> {
        const DEPTH: usize = 32;
        let current_position = &collider.position;
        let mut displacement = collider.physics.clone()?.velocity?;
        collider.position = current_position - &displacement;
        for _ in 0..DEPTH {
            displacement = &displacement / 2.0;
            collider.position = &collider.position + &displacement;
            if collider.intersects(obstacle) {
                collider.position = &collider.position - &displacement;
            }
        }
        Some(collider.position.clone())
    }
}
struct CollisionEntity {
    collider: Collider,
    position: math::Vec3,
    physics: Option<RigidBody>,
}
impl CollisionEntity {
    pub fn intersects(&self, other: &CollisionEntity) -> bool {
        let collider = self.collider.translate(&self.position);
        let other = other.collider.translate(&other.position);
        collider.intersects(&other)
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    vertices: Vec<math::Vec3>,
}
impl Collider {
    pub fn new(vertices: Vec<math::Vec3>) -> Self {
        Self { vertices }
    }

    pub fn line(position: math::Vec3, direction: &math::Vec3) -> Self {
        let end = &position + direction;
        Self {
            vertices: vec![position, end],
        }
    }

    fn translate(&self, position: &math::Vec3) -> Self {
        let vertices: Vec<math::Vec3> = self.vertices.iter().map(|v| v + position).collect();
        Collider { vertices }
    }
    pub fn intersects(&self, other: &Collider) -> bool {
        let initial_dir = vec3!(1.0, 0.0, 0.0);
        let initial_point = self.support(other, &initial_dir);
        let mut simplex = vec![initial_point.clone()];
        let mut direction = -initial_point;
        loop {
            let point = self.support(other, &direction);
            if point == vec3!(0.0) {
                return true;
            }
            if point.dot(&direction) < 0.0 {
                return false; // point lies opposite of search direction so can't be colliding
            }
            simplex.push(point);
            match Self::nearest_simplex(&mut simplex) {
                NearestSimplex::Next { direction: next } => {
                    direction = next;
                }
                NearestSimplex::ContainsOrigin => return true,
            }
        }
    }

    fn support(&self, other: &Collider, direction: &math::Vec3) -> math::Vec3 {
        let left = self.find_furthest_point(direction);
        let right = other.find_furthest_point(&-direction);
        left - right
    }

    fn find_furthest_point(&self, direction: &math::Vec3) -> &math::Vec3 {
        let mut max = f32::NEG_INFINITY;
        let mut index = 0;
        for (v, vertex) in self.vertices.iter().enumerate() {
            let dot = vertex.dot(direction);
            if dot > max {
                index = v;
                max = dot;
            }
        }
        &self.vertices[index]
    }

    fn nearest_simplex(simplex: &mut Vec<math::Vec3>) -> NearestSimplex {
        match simplex.len() {
            2 => Self::intersects_line(simplex),
            3 => Self::intersects_triangle(simplex),
            _ => panic!(
                "collision detection is trying to break into {}th dimension!",
                simplex.len()
            ),
        }
    }

    fn intersects_line(simplex: &[math::Vec3]) -> NearestSimplex {
        let a = &simplex[1];
        let b = &simplex[0];
        let ab = b - a;
        let ao = -a;
        let direction = math::vector_triple_product(&ab, &ao, &ab);
        NearestSimplex::Next { direction }
    }

    fn intersects_triangle(simplex: &mut Vec<math::Vec3>) -> NearestSimplex {
        let (a, b, c) = (&simplex[2], &simplex[1], &simplex[0]);
        let ab = b - a;
        let ac = c - a;
        let ao = -a;
        let rab = math::vector_triple_product(&ac, &ab, &ab);
        let rac = math::vector_triple_product(&ab, &ac, &ac);
        if rab.dot(&ao) > 0.0 {
            simplex.remove(0);
            NearestSimplex::Next { direction: rab }
        } else if rac.dot(&ao) > 0.0 {
            simplex.remove(1);
            NearestSimplex::Next { direction: rac }
        } else {
            NearestSimplex::ContainsOrigin
        }
    }
}

enum NearestSimplex {
    ContainsOrigin,
    Next { direction: math::Vec3 },
}

/// Collects all related collision events
/// When two entities collide, they would both trigger a [CollisionEvent]
/// To know which [events](CollisionEvent) are related, like that entity 1
/// collided with entity 2 we collect them in a [CompoundCollisionEvent]
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundCollisionEvent {
    collisions: Vec<CollisionEvent>,
}
impl CompoundCollisionEvent {
    pub fn new(collisions: Vec<CollisionEvent>) -> Self {
        Self { collisions }
    }

    pub fn get_entities_hit_by<'a, T: Component>(
        &self,
        entity_id: usize,
        entities: &'a Entities<T>,
    ) -> Vec<Entity<'a, T>> {
        self.collisions
            .iter()
            .filter(|e| e.entity_id != entity_id)
            .map(|e| entities.get_entity(e.entity_id))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionEvent {
    pub entity_id: usize,
    /// Approximated collision point of entity with another collider
    /// Is [None] if the colliding entity has no [RigidBody] or the entity is [static](RigidBody::is_static)
    pub collision_point: Option<math::Vec3>,
}
