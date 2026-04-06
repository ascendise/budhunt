use crate::assert_float_eq;
use crate::gfx::tests::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

fn setup(spy_renderer: &SpyRenderer) -> RenderSystem {
    let projection = Projection {
        width: 1280.0,
        height: 720.0,
        fov: 45.0,
        near: 0.1,
        far: 100.0,
    };
    let renderer = Box::new(spy_renderer.clone());
    RenderSystem::new(renderer, projection)
}

#[test]
pub fn render_should_pass_objects_to_renderer() {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Player,
        Components::Position(Default::default()),
    ]); // Camera
    entities.create_entity(vec![Components::Position(Default::default())]); // Some random filler
    let expected_model = Model {
        vao: 123,
        shader: 123,
        material: Texture {
            diffuse: 0,
            specular: 0,
            emission: 0,
            shininess: 32.0,
        },
        transform: Default::default(),
        vertices: 3,
        indices: 3,
    };
    entities.create_entity(vec![Components::Model(expected_model.clone())]);
    let expected_light = DirectionalLight {
        shader: 123,
        direction: vec3!(0.0, -1.0, 0.0),
        material: Material {
            ambient: vec3!(1.0),
            diffuse: vec3!(1.0),
            specular: vec3!(1.0),
        },
    };
    let expected_light = Light::Directional(expected_light);
    entities.create_entity(vec![Components::Light(expected_light.clone())]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let frame = spy.frame(0);
    assert_eq!(vec![expected_model], frame.models);
    assert_eq!(vec![expected_light], frame.lights);
}

#[test]
pub fn render_should_transform_models_with_position() {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Player,
        Components::Position(Default::default()),
    ]); // Camera
    let model = Model {
        vao: 123,
        shader: 123,
        material: Texture {
            diffuse: 0,
            specular: 0,
            emission: 0,
            shininess: 32.0,
        },
        transform: Transform {
            position: vec3!(1.0),
            rotation: vec3!(0.0),
        },
        vertices: 3,
        indices: 3,
    };
    let position = Position {
        position: vec3!(5.0),
        direction: vec3!(0.0),
    };
    entities.create_entity(vec![
        Components::Model(model.clone()),
        Components::Position(position),
    ]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let frame = spy.frame(0);
    let model = frame.models.first().expect("Model was not rendered!");
    assert_float_eq!(Vec3 vec3!(6.0), model.transform.position);
}

#[test_case(Input::Scroll(-10.0), 55.0)]
#[test_case(Input::Scroll(10.0), 35.0)]
pub fn render_should_change_fov_on_scroll(scroll: Input, expected_fov: f32) {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Player,
        Components::Position(Default::default()),
    ]);
    let events = Events::empty();
    events.push_event(Event::Input(scroll));
    // Act
    sut.run(&mut entities, &events);
    // Assert
    let frame = spy.frame(0);
    assert_eq!(expected_fov, frame.projection.fov);
}

#[test_case(Input::Scroll(-10.0), RenderSystem::MAX_FOV)]
#[test_case(Input::Scroll(10.0), RenderSystem::MIN_FOV)]
pub fn render_should_clamp_fov_range(scroll: Input, expected_fov: f32) {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    entities.create_entity(vec![
        Components::Player,
        Components::Position(Default::default()),
    ]);
    // Act
    let mut inputs: Vec<Event> = (0..100).map(|_| Event::Input(scroll.clone())).collect();
    let events = Events::empty();
    events.push_events(&mut inputs);
    sut.run(&mut entities, &events);
    // Assert
    let frame = spy.frame(0);
    assert_eq!(expected_fov, frame.projection.fov);
}
