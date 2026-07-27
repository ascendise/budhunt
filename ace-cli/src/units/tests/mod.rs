use pretty_assertions::assert_eq;
use test_case::test_case;

use crate::units::{Spheric, UV};
use ace::{assert_float_eq, vec3};
use std::f32::consts::PI;

#[test_case(0.0, 0.0)]
#[test_case(0.5, 0.0)]
#[test_case(0.5, 0.5)]
#[test_case(0.0, 0.5)]
#[test_case(1.0, 0.5)]
#[test_case(1.0, 1.0)]
#[test_case(0.0, 1.0)]
pub fn uv_to_spheric_should_return_same_result_after_roundtrip(u: f32, v: f32) {
    // Arrange
    let uv = UV { u, v };
    // Act
    let spheric = uv.to_spheric();
    let new_uv = UV::from_spheric(&spheric);
    // Assert
    assert_eq!(
        uv, new_uv,
        "UV coordinates changed after conversion from spheric!"
    );
}

#[test_case(0.0, 0.0)]
#[test_case(0.5, 0.0)]
#[test_case(0.5, 0.5)]
#[test_case(0.0, 0.5)]
#[test_case(1.0, 0.5)]
#[test_case(1.0, 1.0)]
#[test_case(0.0, 1.0)]
pub fn uv_to_spheric_should_return_result_in_certain_range(u: f32, v: f32) {
    // Arrange
    let uv = UV { u, v };
    // Act
    let spheric = uv.to_spheric();
    // Assert
    assert!(
        spheric.azimuth >= -PI && spheric.azimuth <= PI,
        "azimuth '{}' not in range [-PI, PI]!",
        spheric.azimuth
    );
    assert!(
        spheric.inclination >= 0.0 && spheric.inclination <= PI,
        "inclination '{}' not in range [0, PI]!",
        spheric.inclination
    );
}

#[test_case(0.1, PI - 0.1)]
#[test_case(-PI + 0.1 , PI / 2.1)]
#[test_case(0.1, PI / 2.1)]
#[test_case(PI - 0.1, PI - 0.1)]
pub fn spheric_to_cartesian_should_return_same_result_after_roundtrip(
    azimuth: f32,
    inclination: f32,
) {
    // Arrange
    let spheric = Spheric {
        azimuth,
        inclination,
    };
    // Act
    let cartesian = spheric.to_cartesian();
    let new_spheric = Spheric::from_cartesian(&cartesian);
    // Assert
    assert_float_eq!(spheric.azimuth, new_spheric.azimuth);
    assert_float_eq!(spheric.inclination, new_spheric.inclination);
}

#[test_case(vec3!(1.0, 0.0, 0.0) ; "left")]
#[test_case(vec3!(-1.0, 0.0, 0.0) ; "right")]
#[test_case(vec3!(0.0, 1.0, 0.0) ; "up")]
#[test_case(vec3!(0.0, -1.0, 0.0) ; "down")]
#[test_case(vec3!(0.0, 0.0, 1.0) ; "forward")]
#[test_case(vec3!(0.0, 0.0, -1.0) ; "back")]
#[test_case(vec3!(1.0, 1.0, 0.0))]
#[test_case(vec3!(1.0, 1.0, 1.0))]
#[test_case(vec3!(0.0, 1.0, 1.0))]
pub fn cartesian_to_spheric_should_have_result_in_certain_range(cartesian: ace::math::Vec3) {
    // Arrange
    let cartesian = cartesian.normalize();
    // Act
    let spheric = Spheric::from_cartesian(&cartesian);
    // Assert
    assert!(
        spheric.azimuth >= -PI && spheric.azimuth <= PI,
        "azimuth '{}' not in range [-PI, PI]!",
        spheric.azimuth
    );
    assert!(
        spheric.inclination >= 0.0 && spheric.inclination <= PI,
        "inclination '{}' not in range [0, PI]!",
        spheric.inclination
    );
}
