use std::path::Path;

use crate::gfx::{self, MeshNode};
use pretty_assertions::assert_eq;

#[test]
pub fn load_glb_file_should_load_valid_file_into_mesh() {
    // Arrange
    // This is a file exported from Blender containing three primitive
    // mesh nodes (cube, cylinder, sphere) with their own material each.
    let test_model = Path::new("./src/gfx/tests/TestModel.glb");
    // Act
    let (mesh, _) = gfx::load_glb_file(test_model);
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
pub fn load_glb_file_should_load_collider() {
    // Arrange
    // This is a file exported from Blender containing two primitives,
    // the model itself and a collider (a node called COLLIDER).
    let test_model = Path::new("./src/gfx/tests/TestModelWithCollider.glb");
    // Act
    let (mesh, collider) = gfx::load_glb_file(test_model);
    // Assert
    assert!(collider.is_some(), "collider was not loaded!");
    assert_eq!(1, mesh.nodes.len());
    assert_node_not_empty(&mesh.nodes[0]);
}
