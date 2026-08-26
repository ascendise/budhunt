use pretty_assertions::assert_eq;
use test_case::test_case;

use crate::assert_float_eq;

#[test_case(1.0, 1.0 ; "simple value")]
#[test_case(-1.0, -1.0 ; "negative simple value")]
#[test_case(5.4321, 5.4321 ; "precise value")]
#[test_case(-5.4321, -5.4321 ; "negative precise value")]
pub fn assert_float_eq_should_not_panic_on_equal_values(left: f32, right: f32) {
    assert_float_eq!(left, right);
}

#[test_case(1.0, 2.0 ; "simple value")]
#[test_case(1.0, -1.0 ; "different sign")]
#[test_case(1.00001, 1.0 ; "minor difference")]
#[test_case(1.234, 1.432)]
#[should_panic]
pub fn assert_float_eq_should_panic_on_nonequal_values(left: f32, right: f32) {
    assert_float_eq!(left, right);
}
