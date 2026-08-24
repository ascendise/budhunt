use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::*;
pub mod opengl;

#[cfg(test)]
mod tests;

pub struct RenderSystem {
    renderer: Box<dyn Renderer>,
    projection: Mutex<gfx::Projection>,
}
impl System for RenderSystem {
    fn run(&self, entities: &mut Entities, events: &Events) {
        let mut projection = self.projection.lock().unwrap();
        let inputs = events.get_events(|e| event!(e, Event::Input));
        Self::handle_inputs(&inputs, &mut projection);
        let camera = Self::find_camera(entities);
        let models: Vec<Renderable> = entities
            .get_entities(Components::MODEL)
            .iter()
            .map(|m| Renderable::Model(Self::get_model(m, entities)))
            .collect();
        let mut lights: Vec<Renderable> = entities
            .get_entities(Components::LIGHT)
            .iter()
            .map(|l| Renderable::Light(Self::get_light(l, entities)))
            .collect();
        let mut renderables = models;
        renderables.append(&mut lights);
        let mut lines: Vec<Renderable> = entities
            .get_entities(Components::LINE)
            .iter()
            .map(|l| Renderable::Line(Self::get_line(l, entities)))
            .collect();
        renderables.append(&mut lines);
        self.renderer.render(&projection, &camera, &renderables);
    }
}
impl RenderSystem {
    pub const MIN_FOV: f32 = 1.0;
    pub const MAX_FOV: f32 = 120.0;

    pub fn new(renderer: Box<dyn Renderer>, projection: gfx::Projection) -> Self {
        Self {
            renderer,
            projection: Mutex::new(projection),
        }
    }

    fn find_camera(entities: &mut Entities) -> Camera {
        let entities = entities
            .get_entities(Components::PLAYER | Components::POSITION | Components::DIRECTION);
        let entity = entities.first().expect("Player not found!");
        gfx::Camera {
            position: component!(&entity[Components::POSITION], Components::Position).clone(),
            direction: component!(&entity[Components::DIRECTION], Components::Direction).clone(),
        }
    }

    fn handle_inputs(inputs: &[Input], projection: &mut gfx::Projection) {
        for input in inputs {
            if let Input::Scroll(scroll) = input {
                let fov = projection.fov + -scroll;
                projection.fov = fov.clamp(Self::MIN_FOV, Self::MAX_FOV);
            }
        }
    }

    fn get_model(model: &Entity<'_, Components>, entities: &Entities) -> Model {
        let position = component!(
            &entities[Components::POSITION][model.id()],
            Some(Components::Position) or &Default::default()
        );
        let mut model = component!(&model[Components::MODEL], Components::Model).clone();
        //TODO: Handle rotation via direction
        model.transform.position = &model.transform.position + position;
        model
    }

    fn get_light(light: &Entity<'_, Components>, entities: &Entities) -> Light {
        let position = component!(
            &entities[Components::POSITION][light.id()],
            Some(Components::Position) or &Default::default()
        );
        let mut light = component!(&light[Components::LIGHT], Components::Light).clone();
        light.transform(position);
        light
    }

    fn get_line(line: &Entity<'_, Components>, entities: &Entities) -> Line {
        let position = component!(
            &entities[Components::POSITION][line.id()],
            Some(Components::Position) or &Default::default()
        );
        let mut line = component!(&line[Components::LINE], Components::Line).clone();
        line.transform.position = &line.transform.position + position;
        line
    }
}
pub trait Renderer {
    fn render(&self, projection: &Projection, camera: &Camera, render_targets: &[Renderable]);
}

#[derive(Debug, PartialEq, Clone)]
pub struct Projection {
    pub width: f32,
    pub height: f32,
    /// degrees
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Projection {
    fn to_projection_matrix(&self) -> math::Matrix4 {
        let aspect_ratio = self.width / self.height;
        math::projection(math::radians(self.fov), aspect_ratio, self.near, self.far)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
pub enum Renderable {
    Model(Model),
    Light(Light),
    Line(Line),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Model {
    pub nodes: Vec<ModelNode>,
    pub transform: Transform,
}
pub type VertexArray = u32;
pub type Shader = u32;
#[derive(Debug, PartialEq, Clone)]
pub struct ModelNode {
    pub vao: VertexArray,
    pub shader: Shader,
    pub material: Texture,
    pub vertices: i32,
    pub indices: i32,
}

pub type Tex = i32;
#[derive(PartialEq, Debug, Clone)]
pub struct Texture {
    albedo: Tex,
    metallic_roughness_ao: Tex,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Transform {
    pub position: math::Vec3,
    /// radians
    pub rotation: math::Matrix4, //TODO: Turn into enum Rotation{Eular(Angles), FirstPerson(CameraDirection)}?
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: math::Matrix4::new(1.0),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Light {
    Point(PointLight),
}
impl Light {
    pub fn transform(&mut self, position: &math::Vec3) {
        match self {
            Light::Point(point_light) => point_light.position = &point_light.position + position,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Material {
    pub ambient: math::Vec3,
    pub diffuse: math::Vec3,
    pub specular: math::Vec3,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PointLight {
    pub model: Option<Model>,
    pub color: math::Vec3,
    pub position: math::Vec3,
}

pub struct MeshMeta {
    pub collider: Option<Vec<math::Vec3>>,
    pub points: Vec<math::Vec3>,
}
const COLLIDER_NODE_NAME: &str = "COLLIDER";
const POINT_NODE_PREFIX: &str = "POINT";

/// Reads glb file from file system and returns a [Mesh] consisting of all the meshes inside the glb.
/// If a [gltf::Mesh] has the [gltf::Mesh::name] "COLLIDER", it get's returned as [MeshMeta::collider]
///
/// If a [gltf::Node] has the prefix [gltf::Node::name] "POINT", it's translation get's read and
/// returned with all other points in [MeshMeta::points]
///
/// NOTE: In Blender, make sure the MESH (green symbol) is called "COLLIDER/POINT",
/// not just the OBJECT (orange symbol, parent of mesh).
pub fn load_mesh_from_glb(gltf_path: &std::path::Path) -> (Mesh, MeshMeta) {
    let (document, buffers, images) = gltf::import(gltf_path).unwrap();
    let mut collider = None;
    let mut points = vec![];
    let nodes: Vec<MeshNode> = document
        .nodes()
        .filter_map(|node| {
            let mesh = node.mesh();
            if let Some(mesh) = mesh {
                if mesh.name() == Some(COLLIDER_NODE_NAME) {
                    //TODO: Dont panic! Talk about it instead :D
                    let collider_mesh =
                        load_collider_mesh(&mesh, &buffers).expect("failed to load collider mesh");
                    collider = Some(collider_mesh);
                    None
                } else {
                    let mesh_node =
                        load_mesh_node(&mesh, &buffers, &images).expect("failed to load mesh");
                    Some(mesh_node)
                }
            } else {
                if let Some(name) = node.name()
                    && name.starts_with(POINT_NODE_PREFIX)
                {
                    let point =
                        load_point(&node).unwrap_or_else(|| panic!("failed to load point {name}"));
                    points.push(point);
                }
                None
            }
        })
        .collect();
    if nodes.is_empty() {
        panic!("No model found!")
    }
    let mesh = Mesh { nodes };
    let metainfo = MeshMeta { collider, points };
    (mesh, metainfo)
}

fn load_collider_mesh(
    mesh: &gltf::Mesh<'_>,
    buffers: &[gltf::buffer::Data],
) -> Option<Vec<math::Vec3>> {
    let primitives: Vec<_> = mesh.primitives().collect();
    let primitive = primitives.first()?;
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions: Vec<math::Vec3> = reader
        .read_positions()?
        .map(|v| vec3!(v[0], v[1], v[2]))
        .collect();
    let indices = reader.read_indices()?.into_u32();
    let mesh = indices.map(|i| positions[i as usize].clone()).collect();
    Some(mesh)
}

fn load_point(node: &gltf::Node<'_>) -> Option<math::Vec3> {
    if let gltf::scene::Transform::Decomposed {
        translation,
        rotation: _,
        scale: _,
    } = node.transform()
    {
        Some(vec3!(translation[0], translation[1], translation[2]))
    } else {
        None
    }
}

fn load_mesh_node(
    mesh: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
) -> Option<MeshNode> {
    let primitives: Vec<_> = mesh.primitives().collect();
    let primitive = primitives.first()?;
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .expect("No positions found")
        .collect();
    let normals: Vec<[f32; 3]> = reader.read_normals().expect("No normals found").collect();
    let tex_coords: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .expect("Missing tex coords")
        .into_f32()
        .collect();
    let vertices = read_vertices(positions, normals, tex_coords);
    let indices: Vec<u32> = reader
        .read_indices()
        .expect("No indices found")
        .into_u32()
        .collect();
    let material = primitive.material();
    let pbr = material.pbr_metallic_roughness();
    let albedo = pbr
        .base_color_texture()
        .expect("no albedo texture found")
        .texture();
    let albedo = read_texture(images, albedo);
    let metallic_roughness_ao = pbr
        .metallic_roughness_texture()
        .expect("no metallic-roughness texture found")
        .texture();
    let metallic_roughness_ao = read_texture(images, metallic_roughness_ao);
    let mesh = MeshNode {
        vertices,
        indices,
        albedo,
        metallic_roughness_ao,
    };
    Some(mesh)
}

fn read_vertices(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
) -> Vec<Vertex> {
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
    vertices
}

fn read_texture(images: &[gltf::image::Data], texture: gltf::Texture<'_>) -> Image {
    let texture = texture.source();
    let texture = &images[texture.index()];
    Image {
        data: texture.pixels.clone(),
        width: texture.width,
        height: texture.height,
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub transform: Transform,
    pub shader: Shader,
}

pub struct Mesh {
    pub nodes: Vec<MeshNode>,
}

#[derive(Debug, Clone)]
pub struct MeshNode {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub albedo: Image,
    pub metallic_roughness_ao: Image,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: math::Vec3,
    pub normal: math::Vec3,
    pub texture: math::Vec2,
}

/// Vertex index
pub type Index = u32;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image<P = u8> {
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing, skip_deserializing)]
    pub data: Vec<P>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Ibl {
    pub skybox: Image<f32>,
    pub diffuse: Image<f32>,
    pub brdf_lut: Image,
    pub specular: Vec<Image<f32>>,
}
impl Ibl {
    const SKYBOX_IMAGE_INDEX: usize = 0;
    const DIFFUSE_IMAGE_INDEX: usize = 1;
    const LUT_IMAGE_INDEX: usize = 2;
    const SPECULAR_IMAGE_INDEX: usize = 3;
    const DELIMITER: &[u8; 8] = b"cafebabe";

    pub fn serialize(&self) -> Vec<u8> {
        let data = serde_json::json!(self);
        let mut data = data.to_string().into_bytes();
        data.append(&mut Self::DELIMITER.to_vec());
        self.append_image_data_f32(&mut data, &self.skybox);
        self.append_image_data_f32(&mut data, &self.diffuse);
        self.append_image_data(&mut data, &self.brdf_lut.data);
        for specular in &self.specular {
            self.append_image_data_f32(&mut data, specular);
        }
        data
    }

    fn append_image_data_f32(&self, buffer: &mut Vec<u8>, image: &Image<f32>) {
        let data: Vec<u8> = image
            .data
            .iter()
            .flat_map(|u| u.to_bits().to_le_bytes())
            .collect();
        self.append_image_data(buffer, &data);
    }

    fn append_image_data(&self, buffer: &mut Vec<u8>, data: &[u8]) {
        buffer.append(&mut data.to_vec());
        buffer.append(&mut Self::DELIMITER.to_vec());
    }

    pub fn deserialize(data: &[u8]) -> Ibl {
        let (json, image_data) = Self::split_data(data);
        let mut ibl: Ibl = serde_json::from_str(&json).unwrap();
        ibl.skybox.data = Self::u8_to_f32(image_data[Ibl::SKYBOX_IMAGE_INDEX]);
        ibl.diffuse.data = Self::u8_to_f32(image_data[Ibl::DIFFUSE_IMAGE_INDEX]);
        ibl.brdf_lut.data = image_data[Ibl::LUT_IMAGE_INDEX].to_vec();
        for (s, specular) in &mut ibl.specular.iter_mut().enumerate() {
            specular.data = Self::u8_to_f32(image_data[Ibl::SPECULAR_IMAGE_INDEX + s]);
        }
        ibl
    }

    fn u8_to_f32(data: &[u8]) -> Vec<f32> {
        data.chunks(4)
            .map(|byte| u32::from_ne_bytes([byte[0], byte[1], byte[2], byte[3]]))
            .map(f32::from_bits)
            .collect()
    }

    fn split_data(data: &[u8]) -> (String, Vec<&[u8]>) {
        let image_data_offset = Self::find_next_delimiter(data, 0);
        let json = String::from_utf8(data[0..image_data_offset].to_vec())
            .expect("Could not read json from ibl");
        let images = Self::split_images(&data[image_data_offset + Self::DELIMITER.len()..]);
        (json, images)
    }

    fn find_next_delimiter(data: &[u8], offset: usize) -> usize {
        for (w, window) in data[offset..].windows(Self::DELIMITER.len()).enumerate() {
            if window == Self::DELIMITER {
                return w + offset;
            }
        }
        data.len()
    }

    fn split_images(image_data: &[u8]) -> Vec<&[u8]> {
        let mut images = Vec::new();
        let mut last = 0;
        loop {
            let offset = Self::find_next_delimiter(image_data, last);
            if offset == image_data.len() {
                break;
            }
            images.push(&image_data[last..offset]);
            last = offset + Self::DELIMITER.len();
        }
        images
    }
}
