use std::vec;

use pretty_assertions::assert_eq;
use serde_json::json;

use crate::gfx::{Ibl, Image};

#[test]
pub fn serialize_should_return_data_in_ibl_format() {
    // Arrange
    let ibl = Ibl {
        skybox: Image {
            width: 3,
            height: 3,
            data: vec![1.0, 1.0, 1.0, 1.0],
        },
        diffuse: Image {
            width: 2,
            height: 2,
            data: vec![2.0, 2.0, 2.0, 2.0],
        },
        brdf_lut: Image {
            width: 2,
            height: 2,
            data: vec![1, 2, 3, 4],
        },
        specular: vec![Image {
            width: 2,
            height: 2,
            data: vec![3.0, 3.0, 3.0, 3.0],
        }],
    };
    // Act
    let data = ibl.serialize();
    // Assert
    let mut expected_data = json!({
        "skybox": {
            "width": 3,
            "height": 3,
        },
        "diffuse": {
            "width": 2,
            "height": 2
        },
        "specular": [{
            "width": 2,
            "height": 2
        }],
        "brdf_lut": {
            "width": 2,
            "height": 2
        }
    })
    .to_string()
    .into_bytes();
    let mut expected_image_data: Vec<u8> = vec![];
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    fn to_u8(data: &[f32]) -> Vec<u8> {
        data.iter().flat_map(|f| f.to_ne_bytes()).collect()
    }
    expected_image_data.append(&mut to_u8(&ibl.skybox.data));
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut to_u8(&ibl.diffuse.data));
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut ibl.brdf_lut.data.clone());
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut to_u8(&ibl.specular[0].data));
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_data.append(&mut expected_image_data);
    assert_eq!(expected_data, data);
}

#[test]
pub fn deserialize_should_return_ibl_struct_from_valid_data() {
    // Arrange
    let mut input = json!({
        "skybox": {
            "width": 3,
            "height": 3,
        },
        "diffuse": {
            "width": 2,
            "height": 2
        },
        "brdf_lut": {
            "width": 2,
            "height": 2
        },
        "specular": [{
            "width": 2,
            "height": 2
        }],
    })
    .to_string()
    .into_bytes();
    let mut image_data: Vec<u8> = vec![];
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut 1.0f32.to_ne_bytes().to_vec());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut 2.0f32.to_ne_bytes().to_vec());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut vec![1, 2, 3]);
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut 3.0f32.to_ne_bytes().to_vec());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    input.append(&mut image_data);
    // Act
    let ibl = Ibl::deserialize(&input);
    // Assert
    let expected_ibl = Ibl {
        skybox: Image {
            width: 3,
            height: 3,
            data: vec![1.0],
        },
        diffuse: Image {
            width: 2,
            height: 2,
            data: vec![2.0],
        },
        brdf_lut: Image {
            width: 2,
            height: 2,
            data: vec![1, 2, 3],
        },
        specular: vec![Image {
            width: 2,
            height: 2,
            data: vec![3.0],
        }],
    };
    assert_eq!(expected_ibl, ibl);
}
