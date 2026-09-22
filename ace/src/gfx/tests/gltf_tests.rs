use std::path::{Path, PathBuf};

use crate::gfx::{self, LoadGlbError, MeshNode};
use pretty_assertions::assert_eq;

#[test]
pub fn load_mesh_from_glb_should_return_error_when_file_does_not_exist() {
    // Arrange
    let invalid_path = PathBuf::from(String::from("./this-file-does-not-exist.glb"));
    // Act
    let result = gfx::load_mesh_from_glb(&invalid_path);
    // Assert
    let err = result.err().expect("expected an error but got a model!");
    assert_eq!(LoadGlbError::FileNotFound(invalid_path), err);
}

#[test]
pub fn load_mesh_from_glb_should_return_error_when_model_cannot_be_loaded() {
    // Arrange
    let no_glb_file = String::from("./src/lib.rs");
    // Act
    let result = gfx::load_mesh_from_glb(Path::new(&no_glb_file));
    // Assert
    let err = result.err().expect("expected an error but got a model!");
    assert_eq!(LoadGlbError::InvalidFormat, err);
}

#[test]
pub fn load_mesh_from_glb_should_load_valid_file_into_mesh() {
    // Arrange
    // This is a file exported from Blender containing three primitive
    // mesh nodes (cube, cylinder, sphere) with their own material each.
    let test_model = Path::new("./src/gfx/tests/TestModel.glb");
    // Act
    let (mesh, _) = gfx::load_mesh_from_glb(test_model).unwrap();
    // Assert
    assert_eq!(3, mesh.nodes.len());
    assert_node_not_empty(&mesh.nodes[0]);
    assert_node_not_empty(&mesh.nodes[1]);
    assert_node_not_empty(&mesh.nodes[2]);
}

fn assert_node_not_empty(node: &MeshNode) {
    assert!(!node.vertices.is_empty(), "No vertices loaded!");
    assert!(!node.indices.is_empty(), "No indices loaded!");
}

#[test]
pub fn load_mesh_from_glb_should_load_collider() {
    // Arrange
    // This is a file exported from Blender containing two primitives,
    // the model itself, a collider (a node called COLLIDER)
    // and two single vertices (called POINT_TEST{1,2})
    let test_model = Path::new("./src/gfx/tests/TestModelWithCollider.glb");
    // Act
    let (mesh, metainfo) = gfx::load_mesh_from_glb(test_model).unwrap();
    // Assert
    assert!(metainfo.collider.is_some(), "collider was not loaded!");
    assert_eq!(2, metainfo.points.len(), "points were not (all) loaded!");
    assert_eq!(1, mesh.nodes.len());
    assert_node_not_empty(&mesh.nodes[0]);
}
