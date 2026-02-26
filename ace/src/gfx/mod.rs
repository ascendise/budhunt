use crate::{
    math::{self},
    vec2, vec3,
};

pub mod opengl;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub diffuse: Image,
    pub specular: Image,
    pub emission: Image,
    pub shininess: f32,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: math::Vec3,
    pub normal: math::Vec3,
    pub texture: math::Vec2,
}

/// Vertex index
pub type Index = u32;

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
const EMPTY_IMAGE: Image = Image {
    data: vec![],
    width: 0,
    height: 0,
};
impl Image {
    pub const fn empty() -> Self {
        EMPTY_IMAGE
    }
}

pub trait Renderer {
    fn render(&self, projection: &Projection, camera: &Camera, model: &[Model], lights: &[Light]);
}

#[derive(Debug, Clone)]
pub struct Projection {
    pub width: f32,
    pub height: f32,
    /// radians
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Projection {
    fn to_projection_matrix(&self) -> math::Matrix4 {
        let aspect_ratio = self.width / self.height;
        math::projection(self.fov, aspect_ratio, self.near, self.far)
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: math::Vec3,
    pub direction: math::Vec3,
}
impl Camera {
    fn to_view_matrix(&self) -> math::Matrix4 {
        let center = &self.position + &self.direction;
        let up = vec3!(0.0, 1.0, 0.0); // We do not allow the player to rotate on the z-axis so up is fixed
        math::look_at(&self.position, &center, &up)
    }
}

pub type VertexArray = u32;
pub type Shader = u32;
#[derive(Debug, Clone)]
pub struct Model {
    pub vao: VertexArray,
    pub shader: Shader,
    pub material: Texture,
    pub transform: Transform,
    pub vertices: i32,
    pub indices: i32,
}

pub type Tex = i32;
#[derive(Debug, Clone)]
pub struct Texture {
    pub diffuse: Tex,
    pub specular: Tex,
    pub emission: Tex,
    pub shininess: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Transform {
    pub position: math::Vec3,
    /// radians
    pub rotation: math::Vec3,
}

#[derive(Debug, Clone)]
pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
    Spot(SpotLight),
}

#[derive(Debug, Clone)]
pub struct Material {
    pub ambient: math::Vec3,
    pub diffuse: math::Vec3,
    pub specular: math::Vec3,
}

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub shader: Shader,
    pub direction: math::Vec3,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub model: Model,
    pub color: Material,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[derive(Debug, Clone)]
pub struct SpotLight {
    pub shader: Shader,
    pub position: math::Vec3,
    pub direction: math::Vec3,
    /// cosin of radians
    pub inner_cutoff: f32,
    /// cosin of radians
    pub outer_cutoff: f32,
    pub material: Material,
}

pub fn load_glb_file(gltf_path: &std::path::Path, specular: &Image) -> Mesh {
    let (document, buffers, images) = gltf::import(gltf_path).unwrap();
    for scene in document.scenes() {
        for node in scene.nodes() {
            if let Some(mut mesh) = get_mesh(&node, &buffers, &images) {
                mesh.specular = specular.clone();
                return mesh;
            }
        }
    }
    panic!("No model found!")
}

fn get_mesh(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
) -> Option<Mesh> {
    let mesh = node.mesh()?;
    let primitives: Vec<_> = mesh.primitives().collect();
    let primitive = primitives.first()?;
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .expect("No positions found")
        .collect();
    let normals: Vec<[f32; 3]> = reader.read_normals().expect("No normals found").collect();
    let material = primitive.material();
    let tex_coords: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .expect("Missing tex coords")
        .into_f32()
        .collect();
    let mut vertices = Vec::new();
    for (p, position) in positions.iter().enumerate() {
        let normal = normals[p];
        let tex_coord = tex_coords[p];
        let vertex = Vertex {
            position: vec3!(position[0], position[1], position[2]),
            normal: vec3!(normal[0], normal[1], normal[2]),
            texture: vec2!(tex_coord[0], tex_coord[1]),
        };
        vertices.push(vertex);
    }
    let indices: Vec<u32> = reader
        .read_indices()
        .expect("No indices found")
        .into_u32()
        .collect();
    let diffuse = match material.pbr_metallic_roughness().base_color_texture() {
        Some(texture) => {
            let diffuse = &images[texture.texture().index()];
            Image {
                data: diffuse.pixels.clone(),
                width: diffuse.width,
                height: diffuse.height,
            }
        }
        None => {
            println!("INFO: No base color texture found");
            Image::empty()
        }
    };
    let emission = match material.emissive_texture() {
        Some(texture) => {
            let index = texture.texture().index();
            let image = &images[index];
            Image {
                data: image.pixels.clone(),
                width: image.width,
                height: image.height,
            }
        }
        None => {
            println!("INFO: No emission texture found");
            Image::empty()
        }
    };
    let mesh = Mesh {
        vertices,
        indices,
        diffuse,
        specular: Image::empty(),
        emission,
        shininess: 32.0,
    };
    Some(mesh)
}
