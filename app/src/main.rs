use crate::scripts::MovementScript;
use ace::{
    component,
    gfx::{self},
    math::{self},
    script, vec3,
};
use glfw::Context;
use std::{f32, fs};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
mod scripts;

static VERTEX_SHADER_PBR: &str = include_str!("../shaders/pbr.vs.glsl");
static FRAGMENT_SHADER_PBR: &str = include_str!("../shaders/pbr.fs.glsl");
static TONEMAPPING_SHADER: &str = include_str!("../shaders/tonemapping.glsl");
static VERTEX_SHADER_SKYBOX: &str = include_str!("../shaders/skybox.vs.glsl");
static FRAGMENT_SHADER_SKYBOX: &str = include_str!("../shaders/skybox.fs.glsl");

fn box_collider(width: f32, height: f32, depth: f32) -> ace::physics::Collider {
    let vertices = vec![
        vec3!(-width, -height, depth),
        vec3!(width, -height, depth),
        vec3!(width, -height, -depth),
        vec3!(-width, -height, -depth),
        vec3!(-width, height, -depth),
        vec3!(-width, height, depth),
        vec3!(width, height, depth),
        vec3!(width, height, -depth),
    ];
    ace::physics::Collider::new(vertices)
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let window = setup_window(&mut glfw);
    let mut renderer = gfx::opengl::OpenGlRenderer::init();
    set_skybox(&mut renderer);
    let shader_program = renderer
        .compile_shader(VERTEX_SHADER_PBR, FRAGMENT_SHADER_PBR, TONEMAPPING_SHADER)
        .expect("Failed to compile model shader");
    let mut entities = ace::Entities::empty();
    let clock = Box::new(ace::glfw_input::GlfwClock::new(glfw.clone()));
    let collider = box_collider(0.5, 2.0, 0.5);
    spawn_player(
        &mut renderer,
        shader_program,
        &mut entities,
        clock.clone(),
        collider,
    );
    spawn_targets(&mut renderer, shader_program, &mut entities);
    spawn_point_lights(&mut entities);
    spawn_floor(&mut renderer, shader_program, &mut entities);
    let window = Arc::new(Mutex::new(window));
    let mut world = setup_world(renderer, entities, clock, &window);
    print_opengl_errors();
    while !window.lock().unwrap().should_close() {
        world.run_frame();
        window.lock().unwrap().swap_buffers();
        glfw.poll_events();
    }
}

fn setup_window(glfw: &mut glfw::Glfw) -> glfw::PWindow {
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let (mut window, _) = glfw
        .create_window(1280, 720, "Budhunt", glfw::WindowMode::Windowed)
        .expect("Failed to create window");
    gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);
    gl::Viewport::load_with(|s| window.get_proc_address(s).unwrap() as *const _);
    window.set_framebuffer_size_callback(|_, w, h| unsafe {
        gl::Viewport(0, 0, w, h);
    });
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.make_current();
    window
}

fn set_skybox(renderer: &mut gfx::opengl::OpenGlRenderer) {
    let skybox = fs::read("./app/skybox.ibl").expect("failed to read skybox.ibl");
    let skybox = gfx::Ibl::deserialize(&skybox);
    let shader = renderer
        .compile_shader(
            VERTEX_SHADER_SKYBOX,
            FRAGMENT_SHADER_SKYBOX,
            TONEMAPPING_SHADER,
        )
        .unwrap();
    renderer.set_skybox(&skybox, shader);
}

fn spawn_targets(
    renderer: &mut gfx::opengl::OpenGlRenderer,
    shader_program: u32,
    entities: &mut ace::Entities,
) -> ace::physics::Collider {
    // Collider should be hitting player even if visible object is very high (for testing)
    let (mesh, collider) = gfx::load_glb_file(Path::new("./app/models/Target.glb"));
    let model = renderer.load_mesh(&mesh, shader_program);
    let targets = [
        vec3!(0.0, 0.0, 0.0),
        vec3!(2.0, 5.0, -15.0),
        vec3!(-1.5, -2.2, -2.5),
        vec3!(-3.8, -2.0, -12.3),
        vec3!(2.4, -0.4, -3.5),
        vec3!(-1.7, 3.0, 7.5),
        vec3!(1.3, -2.0, -2.5),
        vec3!(1.5, 2.0, -2.5),
        vec3!(1.5, 0.2, -1.5),
        vec3!(-1.3, 1.0, -1.5),
    ];
    let collider = ace::physics::Collider::new(collider.expect("no collider found for Target.glb"));
    for target in targets {
        let position = ace::Components::Position(target);
        let collider = ace::Components::Collider(collider.clone());
        let components = vec![ace::Components::Model(model.clone()), position, collider];
        entities.create_entity(components);
    }
    collider
}

fn spawn_point_lights(entities: &mut ace::Entities) {
    let point_lights = [
        vec3!(0.7, 0.2, 2.0),
        vec3!(2.3, -3.3, -4.0),
        vec3!(-4.0, 2.0, -12.0),
        vec3!(0.0, 0.0, -3.0),
    ];
    let move_script = script!(|entity: &ace::Entity<'_, ace::Components>, _| {
        let position = component!(
            &entity[ace::Components::POSITION],
            ace::Components::Position
        );
        let position = position + &vec3!(0.0, 0.001, 0.0);
        vec![ace::Components::Position(position)]
    });
    let move_script = Box::new(move_script);
    for position in point_lights {
        let light = create_point_light(position.clone());
        let light = gfx::Light::Point(light);
        let light = ace::Components::Light(light);
        let position = ace::Components::Position(position);
        let script = ace::Components::Scripts(vec![move_script.clone()]);
        entities.create_entity(vec![light, position, script]);
    }
}
fn create_point_light(position: math::Vec3) -> gfx::PointLight {
    gfx::PointLight {
        model: None,
        color: vec3!(1.0, 1.0, 1.0) * 10.0,
        position,
    }
}

fn spawn_floor(
    renderer: &mut gfx::opengl::OpenGlRenderer,
    shader_program: u32,
    entities: &mut ace::Entities,
) {
    let (mut plane_mesh, _) = gfx::load_glb_file(Path::new("./app/models/Plane.glb"));
    // Scale / Move model programatically
    plane_mesh.nodes[0].vertices = plane_mesh.nodes[0]
        .vertices
        .iter_mut()
        .map(|v| {
            let mut new = v.clone();
            new.position = new.position * 1024.0;
            new.position.y -= 10.0;
            new
        })
        .collect();
    let plane_model = renderer.load_mesh(&plane_mesh, shader_program);
    entities.create_entity(vec![
        ace::Components::Model(plane_model),
        ace::Components::Position(Default::default()),
    ]);
}

fn spawn_player(
    renderer: &mut gfx::opengl::OpenGlRenderer,
    shader_program: gfx::Shader,
    entities: &mut ace::Entities,
    clock: Box<ace::glfw_input::GlfwClock>,
    collider: ace::physics::Collider,
) {
    let (mesh, _) = gfx::load_glb_file(Path::new("./app/models/Rifle.glb"));
    let model = renderer.load_mesh(&mesh, shader_program);
    entities.create_entity(vec![
        //ace::Components::Position(vec3!(0.0, 0.0, -50.0)),
        ace::Components::Position(vec3!(0.0, 0.0, 2.0)),
        ace::Components::Direction(vec3!(0.0, 0.0, 1.0)),
        ace::Components::Scripts(vec![Box::new(MovementScript::new(clock))]),
        ace::Components::Player,
        ace::Components::Collider(collider),
        ace::Components::Model(model),
        ace::Components::RigidBody(ace::physics::RigidBody::new(vec3!(0.0))),
    ]);
}

fn setup_world(
    renderer: gfx::opengl::OpenGlRenderer,
    entities: ace::Entities,
    clock: Box<ace::glfw_input::GlfwClock>,
    window: &Arc<Mutex<glfw::PWindow>>,
) -> ace::World {
    let (width, height) = window.lock().unwrap().get_size();
    let projection = gfx::Projection {
        width: width as f32,
        height: height as f32,
        fov: 75.0,
        near: 0.1,
        far: 1000.0,
    };
    let render_system = Box::new(ace::gfx::RenderSystem::new(Box::new(renderer), projection));
    let script_system = Box::new(ace::scripts::ScriptSystem);
    let physics_system = Box::new(ace::physics::PhysicsSystem::new(Some(
        ace::physics::CollisionSystem,
    )));
    let glfw_inputs = ace::glfw_input::GlfwInputsImpl::new(window.clone());
    let input_listener = ace::glfw_input::GlfwInputListener::init(Box::new(glfw_inputs));
    ace::World::init(
        entities,
        vec![render_system, script_system, physics_system],
        clock.clone(),
        Box::new(input_listener),
    )
}

fn print_opengl_errors() {
    unsafe {
        loop {
            let err = gl::GetError();
            if err == 0 {
                break;
            }
            eprintln!("OPENGL ERROR ({err})");
        }
    }
}
