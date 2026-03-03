#[macro_export]
macro_rules! assert_float_eq {
    (Matrix4 $left:expr, $right:expr) => {{
        let mut not_equal = false;
        for i in 0..=3 {
            for y in 0..=3 {
                if !$crate::float_is_near!($left.data[i][y], $right.data[i][y]) {
                    not_equal = true;
                    break;
                }
            }
        }
        if not_equal {
            assert_eq!($left, $right, "Matrix4 do not match!");
        }
    }};
    (Vec4 $left:expr, $right:expr) => {{
        if !($crate::float_is_near!($left.x, $right.x)
            && $crate::float_is_near!($left.y, $right.y)
            && $crate::float_is_near!($left.z, $right.z)
            && $crate::float_is_near!($left.w, $right.w))
        {
            assert_eq!($left, $right);
        }
    }};
    (Vec3 $left:expr, $right:expr) => {{
        if !($crate::float_is_near!($left.x, $right.x)
            && $crate::float_is_near!($left.y, $right.y)
            && $crate::float_is_near!($left.z, $right.z))
        {
            assert_eq!($left, $right);
        }
    }};
    (Vec2 $left:expr, $right:expr) => {{
        if !($crate::float_is_near!($left.x, $right.x) && $crate::float_is_near!($left.y, $right.y))
        {
            assert_eq!($left, $right);
        }
    }};
    ($left:expr, $right:expr) => {{
        if !$crate::float_is_near!($left, $right) {
            assert_eq!($left, $right)
        }
    }};
}
// https://web.archive.org/web/20251227112838/https://floating-point-gui.de/errors/comparison/
#[macro_export]
macro_rules! float_is_near {
    ($left:expr, $right:expr) => {
        if $left == $right {
            true
        } else {
            let left_abs = $left.abs();
            let right_abs = $right.abs();
            let diff = (left_abs - right_abs).abs();
            if $left == 0.0 || $right == 0.0 || left_abs + right_abs < f32::MIN_POSITIVE {
                diff < f32::EPSILON
            } else {
                diff / f32::min(left_abs + right_abs, f32::MAX) < f32::EPSILON
            }
        }
    };
}
