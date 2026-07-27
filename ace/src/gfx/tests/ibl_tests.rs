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
            data: "skyboximg".to_string().into_bytes(),
        },
        diffuse: Image {
            width: 2,
            height: 2,
            data: "helo".to_string().into_bytes(),
        },
        brdf_lut: Image {
            width: 2,
            height: 2,
            data: "cafe".to_string().into_bytes(),
        },
        specular: vec![Image {
            width: 2,
            height: 2,
            data: "babe".to_string().into_bytes(),
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
    expected_image_data.append(&mut ibl.skybox.data.clone());
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut ibl.diffuse.data.clone());
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut ibl.brdf_lut.data.clone());
    expected_image_data.append(&mut Ibl::DELIMITER.to_vec());
    expected_image_data.append(&mut ibl.specular[0].data.clone());
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
    image_data.append(&mut String::from("skyboximg").into_bytes());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut String::from("helo").into_bytes());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut String::from("cafe").into_bytes());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    image_data.append(&mut String::from("babe").into_bytes());
    image_data.append(&mut Ibl::DELIMITER.to_vec());
    input.append(&mut image_data);
    // Act
    let ibl = Ibl::deserialize(&input);
    // Assert
    let expected_ibl = Ibl {
        skybox: Image {
            width: 3,
            height: 3,
            data: "skyboximg".to_string().into_bytes(),
        },
        diffuse: Image {
            width: 2,
            height: 2,
            data: "helo".to_string().into_bytes(),
        },
        brdf_lut: Image {
            width: 2,
            height: 2,
            data: "cafe".to_string().into_bytes(),
        },
        specular: vec![Image {
            width: 2,
            height: 2,
            data: "babe".to_string().into_bytes(),
        }],
    };
    assert_eq!(expected_ibl, ibl);
}
