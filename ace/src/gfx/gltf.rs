use super::*;
use ::gltf::animation::util::ReadOutputs;

//TODO: move into gltf module (with feature flag? do same for opengl?)
//TODO: dont mention glb in naming, gltf/glb can be treated synonymous and gltf must be preferred

/// Reads glb file from file system and returns a [Mesh] consisting of all the meshes inside the glb.
/// If a [gltf::Mesh] has the [gltf::Mesh::name] "COLLIDER", it get's returned as [MeshMeta::collider]
///
/// If a [gltf::Node] has the prefix [gltf::Node::name] "POINT", it's translation get's read and
/// returned with all other points in [MeshMeta::points]
///
/// NOTE: In Blender, make sure the MESH (green symbol) is called "COLLIDER/POINT",
/// not just the OBJECT (orange symbol, parent of mesh).
pub fn load_mesh_from_glb(gltf_path: &std::path::Path) -> Result<(Mesh, MeshMeta), LoadGlbError> {
    let (document, buffers, images) = import_gltf(gltf_path)?;
    let (collider, points, nodes) = load_mesh_data(&document, &buffers, &images);
    let animations = load_animations(document, buffers);
    let mesh = Mesh { nodes, animations };
    let metainfo = MeshMeta { collider, points };
    Ok((mesh, metainfo))
}

fn import_gltf(
    gltf_path: &std::path::Path,
) -> Result<
    (
        ::gltf::Document,
        Vec<::gltf::buffer::Data>,
        Vec<::gltf::image::Data>,
    ),
    LoadGlbError,
> {
    match ::gltf::import(gltf_path) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            ::gltf::Error::Deserialize(_) => Err(LoadGlbError::InvalidFormat),
            ::gltf::Error::Io(_) => Err(LoadGlbError::FileNotFound(gltf_path.into())),
            _ => Err(LoadGlbError::Corrupted(e.to_string())),
        },
    }
}
fn load_mesh_data(
    document: &::gltf::Document,
    buffers: &[::gltf::buffer::Data],
    images: &[::gltf::image::Data],
) -> (Option<Vec<math::Vec3>>, Vec<math::Vec3>, Vec<MeshNode>) {
    let mut collider = None;
    let mut points = vec![];
    let mut nodes = vec![];
    for node in document.nodes() {
        let mesh = node.mesh();
        if let Some(mesh) = mesh {
            if mesh.name() == Some(COLLIDER_NODE_NAME) {
                let mesh = load_collider(&mesh, buffers).expect("failed to load collider mesh");
                collider = Some(mesh);
            } else {
                let mesh_node = load_mesh(&mesh, buffers, images).expect("failed to load mesh");
                nodes.push(mesh_node);
            }
        } else {
            if let Some(name) = node.name()
                && name.starts_with(POINT_NODE_PREFIX)
            {
                let point =
                    load_point(&node).unwrap_or_else(|| panic!("failed to load point {name}"));
                points.push(point);
            }
        }
    }
    if nodes.is_empty() {
        panic!("No model found!")
    }
    (collider, points, nodes)
}

fn load_collider(
    mesh: &::gltf::Mesh<'_>,
    buffers: &[::gltf::buffer::Data],
) -> Option<Vec<math::Vec3>> {
    let primitives: Vec<_> = mesh.primitives().collect();
    let primitive = primitives.first()?;
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions: Vec<math::Vec3> = reader
        .read_positions()?
        .map(|v| vec3!(v[0], v[1], v[2]))
        .collect();
    let indices = reader.read_indices()?.into_u32();
    let mesh = indices.map(|i| positions[i as usize]).collect();
    Some(mesh)
}

fn load_point(node: &::gltf::Node<'_>) -> Option<math::Vec3> {
    if let ::gltf::scene::Transform::Decomposed {
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

fn load_mesh(
    mesh: &::gltf::Mesh,
    buffers: &[::gltf::buffer::Data],
    images: &[::gltf::image::Data],
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
    let indices = reader.read_indices();
    let indices = if let Some(indices) = indices {
        let vec: Vec<u32> = indices.into_u32().collect();
        Some(vec)
    } else {
        None
    };
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

fn read_texture(images: &[::gltf::image::Data], texture: ::gltf::Texture<'_>) -> Image {
    let texture = texture.source();
    let texture = &images[texture.index()];
    Image {
        data: texture.pixels.clone(),
        width: texture.width,
        height: texture.height,
    }
}

fn load_animations(
    document: ::gltf::Document,
    buffers: Vec<::gltf::buffer::Data>,
) -> Vec<super::Animation> {
    let mut animations = vec![];
    for animation in document.animations() {
        let mut channels = vec![];
        for channel in animation.channels() {
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let inputs: Vec<f32> = reader.read_inputs().unwrap().collect();
            let outputs = reader.read_outputs().unwrap();
            let transforms = read_transforms(outputs);
            channels.push(Channel {
                timestamps: inputs,
                transforms,
                node: channel.target().node().index(),
            });
        }
        animations.push(Animation(channels));
    }
    animations;
    todo!()
}

fn read_transforms(outputs: ReadOutputs<'_>) -> Transform {
    match outputs {
        ReadOutputs::Translations(translations) => {
            Transform::Translations(translations.map(math::Vec3::from).collect())
        }
        ReadOutputs::Rotations(rotations) => {
            Transform::Rotations(rotations.into_f32().map(math::Vec4::from).collect())
        }
        ReadOutputs::Scales(scales) => Transform::Scales(scales.map(math::Vec3::from).collect()),
        ReadOutputs::MorphTargetWeights(_) => {
            panic!("the fuck?")
        }
    }
}

struct Animation(Vec<Channel>);
struct Channel {
    timestamps: Vec<f32>,
    transforms: Transform,
    node: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    Translations(Vec<math::Vec3>),
    Rotations(Vec<math::Vec4>), //TODO: actually Quaternions
    Scales(Vec<math::Vec3>),
}
