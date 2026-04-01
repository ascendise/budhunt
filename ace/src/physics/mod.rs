use crate::{Components, Position, System, component, math, vec3, vec4};

#[cfg(test)]
mod tests;

pub struct CollisionSystem;
impl System for CollisionSystem {
    fn run(&self, entities: &mut crate::Entities, _: &[crate::Input]) {
        let colliders = entities.get_bucket(Components::COLLIDER);
        let positions = entities.get_bucket(Components::POSITION);
        for (c, collider) in colliders.iter().enumerate().filter(|(_, c)| c.is_some()) {
            let position = component!(&positions[c], Some(Components::Position)
                or &Position::default());
            let collider = component!(collider, Some(Components::Collider)).translate(position);
            for (o, other) in colliders
                .iter()
                .enumerate()
                .skip(c + 1)
                .filter(|(_, c)| c.is_some())
            {
                let other_position = component!(&positions[o], Some(Components::Position)
                    or &Position::default());
                let other = component!(other, Some(Components::Collider)).translate(other_position);
                if collider.intersects(&other) {
                    println!("[{c}] Collision detected with {o}");
                }
            }
        }
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

    fn translate(&self, position: &Position) -> Self {
        let position = &position.position;
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

#[derive(Clone)]
pub struct CollisionEvent(usize, usize);
