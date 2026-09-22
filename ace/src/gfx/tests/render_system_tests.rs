use crate::assert_float_eq;
use crate::gfx::tests::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

fn setup(spy_renderer: &SpyRenderer) -> RenderSystem {
    let projection = Projection {
        width: 1280.0,
        height: 720.0,
        fov: math::radians(45.0),
        near: 0.1,
        far: 100.0,
    };
    let renderer = Box::new(spy_renderer.clone());
    RenderSystem::new(renderer, projection)
}

fn setup_camera(entities: &mut Entities) {
    setup_camera_at(entities, vec3!(0.0));
}

fn setup_camera_at(entities: &mut Entities, position: math::Vec3) {
    entities.create_entity(vec![Components::Player, Transform::new(position).into()]);
}

#[test]
pub fn render_should_pass_objects_to_renderer_with_default_transform() {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    setup_camera(&mut entities);
    entities.create_entity(vec![Components::Transform(Default::default())]); // Some random filler
    let expected_node = ModelNode {
        vao: 123,
        shader: 123,
        material: Texture {
            albedo: 0,
            metallic_roughness_ao: 0,
        },
        vertices: 3,
        indices: 3,
    };
    let expected_nodes = vec![expected_node];
    entities.create_entity(vec![Components::Model(expected_nodes.clone())]);
    let expected_light = PointLight {
        model: Some(expected_nodes.clone()),
        color: vec3!(1.0),
        position: vec3!(1.0),
    };
    let expected_light = Light::Point(expected_light);
    entities.create_entity(vec![Components::Light(expected_light.clone())]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let frame = spy.frame(0);
    let expected_model = Model {
        nodes: expected_nodes,
        transform: Default::default(),
    };
    assert_eq!(vec![expected_model], frame.models);
    assert_eq!(vec![expected_light], frame.lights);
}

#[test]
pub fn render_should_transform_models_with_specific_transform() {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    setup_camera(&mut entities);
    let model = ModelNode {
        vao: 123,
        shader: 123,
        material: Texture {
            albedo: 0,
            metallic_roughness_ao: 0,
        },
        vertices: 3,
        indices: 3,
    };
    let expected_transform = Transform {
        position: vec3!(5.0),
        rotation: math::rotation(&vec3!(1.0)),
    };
    let model = vec![model];
    entities.create_entity(vec![
        Components::Model(model),
        Components::Transform(expected_transform),
    ]);
    // Act
    sut.run(&mut entities, &Events::empty());
    // Assert
    let frame = spy.frame(0);
    let model = frame.models.first().expect("Model was not rendered!");
    assert_eq!(
        expected_transform, model.transform,
        "did not pass correct transforms!"
    );
}

#[test_case(Input::Scroll(-10.0), math::radians(55.0))]
#[test_case(Input::Scroll(10.0), math::radians(35.0))]
pub fn render_should_change_fov_on_scroll(scroll: Input, expected_fov: f32) {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    setup_camera(&mut entities);
    let events = Events::empty();
    events.push_event(Event::Input(scroll));
    // Act
    sut.run(&mut entities, &events);
    // Assert
    let frame = spy.frame(0);
    assert_float_eq!(expected_fov, frame.projection.fov);
}

#[test_case(Input::Scroll(-10.0), RenderSystem::MAX_FOV)]
#[test_case(Input::Scroll(10.0), RenderSystem::MIN_FOV)]
pub fn render_should_clamp_fov_range(scroll: Input, expected_fov: f32) {
    // Arrange
    let spy = SpyRenderer::new();
    let sut = setup(&spy);
    let mut entities = Entities::empty();
    setup_camera(&mut entities);
    // Act
    let mut inputs: Vec<Event> = (0..100).map(|_| Event::Input(scroll.clone())).collect();
    let events = Events::empty();
    events.push_events(&mut inputs);
    sut.run(&mut entities, &events);
    // Assert
    let frame = spy.frame(0);
    assert_float_eq!(expected_fov, frame.projection.fov);
}
