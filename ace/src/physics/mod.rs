use crate::{
    Components, Entities, Entity, Event, Events, System, component, event, math, maybe_component,
    vec3, vec4,
};

#[cfg(test)]
mod tests;

pub struct PhysicsSystem {
    collision_system: Option<CollisionSystem>,
}
impl System for PhysicsSystem {
    fn run(&self, entities: &mut Entities, events: &Events) {
        let mut new_positions = vec![];
        for entity in entities.get_entities(Components::RIGIDBODY | Components::POSITION) {
            Self::move_entity(&entity, &mut new_positions);
        }
        for (e, new_position) in new_positions {
            entities.update_entity(e, Components::Position(new_position));
        }
        if let Some(collision_system) = &self.collision_system {
            Self::handle_collisions(entities, events, collision_system);
        }
    }
}
impl PhysicsSystem {
    pub fn new(collision_system: Option<CollisionSystem>) -> Self {
        Self { collision_system }
    }

    fn move_entity(entity: &Entity<'_, Components>, new_positions: &mut Vec<(usize, math::Vec3)>) {
        let rigid_body = component!(&entity[Components::RIGIDBODY], Components::RigidBody);
        if let Some(velocity) = &rigid_body.velocity {
            let position = component!(&entity[Components::POSITION], Components::Position);
            new_positions.push((entity.id, position + velocity));
        }
    }

    fn handle_collisions(
        entities: &mut Entities,
        events: &Events,
        collision_system: &CollisionSystem,
    ) {
        collision_system.run(entities, events);
        for event in events.get_events(|e| event!(e, Event::Collision)) {
            if let Some(collision_point) = event.collision_point {
                entities.update_entity(event.entity_id, Components::Position(collision_point));
            }
        }
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
        let colliders = entities.get_entities(Components::COLLIDER | Components::POSITION);
        let rigid_bodies = entities.get_bucket(Components::RIGIDBODY);
        for collider in &colliders {
            let mut collision_entity = CollisionEntity {
                collider: component!(&collider[Components::COLLIDER], Components::Collider).clone(),
                position: component!(&collider[Components::POSITION], Components::Position).clone(),
                physics: maybe_component!(&rigid_bodies[collider.id()], Components::RigidBody)
                    .cloned(),
            };
            for other in colliders.iter().skip(collider.id() + 1) {
                let mut other_collision_entity = CollisionEntity {
                    collider: component!(&other[Components::COLLIDER], Components::Collider)
                        .clone(),
                    position: component!(&other[Components::POSITION], Components::Position)
                        .clone(),
                    physics: maybe_component!(&rigid_bodies[other.id()], Components::RigidBody)
                        .cloned(),
                };
                if collision_entity.intersects(&other_collision_entity) {
                    let collider_collision_point =
                        Self::find_collision_point(&mut collision_entity, &other_collision_entity);
                    let obstacle_collision_point =
                        Self::find_collision_point(&mut other_collision_entity, &collision_entity);
                    events.push_events(&mut vec![
                        Event::Collision(CollisionEvent {
                            entity_id: collider.id(),
                            collision_point: collider_collision_point,
                        }),
                        Event::Collision(CollisionEvent {
                            entity_id: other.id(),
                            collision_point: obstacle_collision_point,
                        }),
                    ]);
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
    pub const SIMPLEX_DIRS: [math::Vec3; 4] = [
        vec3!(1.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(-1.0, 0.0, 0.0),
    ];

    pub fn new(vertices: Vec<math::Vec3>) -> Self {
        Self { vertices }
    }

    fn translate(&self, position: &math::Vec3) -> Self {
        let vertices: Vec<math::Vec3> = self.vertices.iter().map(|v| v + position).collect();
        Collider { vertices }
    }

    pub fn intersects(&self, other: &Collider) -> bool {
        let initial_dir = &Self::SIMPLEX_DIRS[0];
        let origin = self.support(initial_dir) - other.support(&-initial_dir);
        let mut simplex = vec![origin];
        for direction in &Self::SIMPLEX_DIRS[1..] {
            let point = self.support(direction) - other.support(&-direction);
            if point.dot(direction) < 0.0 {
                return false;
            }
            simplex.push(point);
            if Self::nearest_simplex(&simplex) {
                return true;
            }
        }
        false
    }

    fn support(&self, direction: &math::Vec3) -> &math::Vec3 {
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

    fn nearest_simplex(simplex: &[math::Vec3]) -> bool {
        match simplex.len() {
            1 => simplex[0] == vec3!(0.0),
            2 => Self::intersects_line(simplex),
            3 => Self::intersects_triangle(simplex),
            _ => Self::intersects_tetrahedron(simplex),
        }
    }

    fn intersects_line(line: &[math::Vec3]) -> bool {
        let point1 = &line[0];
        let point2 = &line[1];
        (-point1).cross(&(-point2)) == vec3!(0.0)
    }

    fn intersects_triangle(plane: &[math::Vec3]) -> bool {
        let (point1, point2, point3) = (&plane[0], &plane[1], &plane[2]);
        let denominator = (point2 - point1).cross(&(point3 - point1)).magnitude() / 2.0;
        let alpha = point2.cross(point3).magnitude() / (2.0 * denominator);
        if !f32_in_range(alpha, 0.0, 1.0) {
            return false;
        }
        let beta = point3.cross(point1).magnitude() / (2.0 * denominator);
        if !f32_in_range(beta, 0.0, 1.0) {
            return false;
        }
        let gamma = 1.0 - alpha - beta;
        f32_in_range(gamma, 0.0, 1.0)
    }

    fn intersects_tetrahedron(polygon: &[math::Vec3]) -> bool {
        let tetrahedron: math::Matrix4 = [
            [polygon[0].x, polygon[1].x, polygon[2].x, polygon[3].x],
            [polygon[0].y, polygon[1].y, polygon[2].y, polygon[3].y],
            [polygon[0].z, polygon[1].z, polygon[2].z, polygon[3].z],
            [1.0, 1.0, 1.0, 1.0],
        ]
        .into();
        let origin = vec4!(0.0, 0.0, 0.0, 1.0);
        let barycentric_coords = tetrahedron.inverse() * origin;
        f32_in_range(barycentric_coords.x, 0.0, 1.0)
            && f32_in_range(barycentric_coords.y, 0.0, 1.0)
            && f32_in_range(barycentric_coords.z, 0.0, 1.0)
            && f32_in_range(barycentric_coords.w, 0.0, 1.0)
            && barycentric_coords.w
                == 1.0 - barycentric_coords.x - barycentric_coords.y - barycentric_coords.z
    }
}

fn f32_in_range(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionEvent {
    pub entity_id: usize,
    /// Approximated collision point of entity with another collider
    /// Is [None] if the colliding entity has no [RigidBody] or the entity is [static](RigidBody::is_static)
    pub collision_point: Option<math::Vec3>,
}
