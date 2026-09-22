// No actual asserts run here.
// Just some Component definitions that should not be compiled into the library.
// Those component definitions define edge cases the derive macro should be able to handle.
use ace::Component;
use ace_proc_macros::Component;

/// simple component should be generated
#[allow(unused)]
#[derive(Component)]
enum SimpleCompnonent {
    Number(u32),
    Float(f32),
    Bool(bool),
    Marker1,
    Marker2,
}

/// should skip generating From for variants sharing the same type
#[allow(unused)]
#[derive(Component)]
enum SameTypeVariantComponent {
    Number(u32),
    Float(f32),
    Float2(f32),
}

#[allow(unused)]
#[derive(Component)]
/// should generate constants with 'CONST_' prefix to avoid collision with the variant
enum SingleCharVariants {
    A,
    B,
    C,
}
#[allow(unused)]
impl SingleCharVariants {
    fn print_consts() {
        println!("{}", SingleCharVariants::CONST_A);
        println!("{}", SingleCharVariants::CONST_B);
        println!("{}", SingleCharVariants::CONST_C);
    }
}
