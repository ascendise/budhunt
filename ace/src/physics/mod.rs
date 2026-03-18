use crate::{math, vec3};

#[cfg(test)]
mod tests;

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
            _ => Self::intersects_triangle(simplex),
            //_ => Self::intersects_tetrahedron(simplex),
        }
    }

    fn intersects_line(line: &[math::Vec3]) -> bool {
        let origin = &vec3!(0.0);
        let point1 = &line[0];
        let point2 = &line[1];
        (point1 - origin) + (point2 - origin) == point1 - point2
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
        todo!()
    }
}

fn f32_in_range(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max
}
