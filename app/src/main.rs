use crate::scripts::MovementScript;
use ace::{
    component,
    gfx::{self, Image},
    math::{self},
    script, vec3,
};
use glfw::Context;
use std::f32;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
mod scripts;

static VERTEX_SHADER_SOURCE: &str = include_str!("../shaders/object.vs.glsl");
static FRAGMENT_SHADER_SOURCE: &str = include_str!("../shaders/object.fs.glsl");

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let window = setup_window(&mut glfw);
    let mut renderer = gfx::opengl::OpenGlRenderer::init();
    let shader_program = renderer
        .compile_shader(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
        .expect("Failed to compile model shader");
    let mut entities = ace::Entities::empty();
    let flashlight = create_spotlight(shader_program);
    let clock = Box::new(ace::glfw_input::GlfwClock::new(glfw.clone()));
    entities.add_entity(vec![
        ace::Component::Light(flashlight),
        ace::Component::Position(Default::default()),
        ace::Component::Scripts(vec![Box::new(MovementScript::new(clock.clone()))]),
    ]); //First entity has to be player/camera
    // Monkey models
    let specular_map = load_image(Path::new(
        "/home/ascendise/dev/budhunt/app/models/suzanne_specular.png",
    ));
    let monkey_mesh = gfx::load_glb_file(
        Path::new("/home/ascendise/dev/budhunt/app/models/Suzanne.glb"),
        &specular_map,
    );
    let monkey_model = renderer.load_mesh(&monkey_mesh, shader_program);
    let monkeys = [
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
    let move_script = script!(|entity: &[&ace::Component], _| {
        let position = entity
            .iter()
            .find(|e| matches!(e, ace::Component::Position(_)));
        let mut position = component!(position, Some(ace::Component::Position)).clone();
        position.position = position.position + vec3!(0.0, 0.001, 0.0);
        vec![ace::Component::Position(position)]
    });
    let move_script = Box::new(move_script);
    for monkey in monkeys {
        let model = monkey_model.clone();
        let position = ace::Component::Position(ace::Position {
            position: monkey,
            direction: Default::default(),
        });
        entities.add_entity(vec![ace::Component::Model(model), position]);
    }
    // Lights
    let light_mesh = gfx::load_glb_file(
        Path::new("/home/ascendise/dev/budhunt/app/models/Light.glb"),
        &Image::empty(),
    );
    let point_light = create_point_light(&mut renderer, &light_mesh, shader_program);
    let point_lights = [
        vec3!(0.7, 0.2, 2.0),
        vec3!(2.3, -3.3, -4.0),
        vec3!(-4.0, 2.0, -12.0),
        vec3!(0.0, 0.0, -3.0),
    ];
    for position in point_lights {
        let light = gfx::Light::Point(point_light.clone());
        let light = ace::Component::Light(light);
        let position = ace::Component::Position(ace::Position {
            position,
            direction: Default::default(),
        });
        let script = ace::Component::Scripts(vec![move_script.clone()]);
        entities.add_entity(vec![light, position, script]);
    }
    let sun_color = vec3!(1.0, 1.0, 1.0);
    let dir_light = gfx::DirectionalLight {
        shader: shader_program,
        direction: vec3!(-0.2, -1.0, -0.3),
        material: gfx::Material {
            ambient: &vec3!(0.2, 0.2, 0.2) * &sun_color,
            diffuse: &vec3!(0.5, 0.5, 0.5) * &sun_color,
            specular: &vec3!(1.0, 1.0, 1.0) * &sun_color,
        },
    };
    let dir_light = ace::Component::Light(gfx::Light::Directional(dir_light));
    entities.add_entity(vec![dir_light]);
    // Plane
    let plane_specular = load_image(Path::new(
        "/home/ascendise/dev/budhunt/app/models/plane_specular.png",
    ));
    let mut plane_mesh = gfx::load_glb_file(
        Path::new("/home/ascendise/dev/budhunt/app/models/Plane.glb"),
        &plane_specular,
    );
    // Scale / Move model programatically
    plane_mesh.vertices = plane_mesh
        .vertices
        .iter_mut()
        .map(|v| {
            let mut new = v.clone();
            new.position = new.position * 1024.0;
            new.position.y -= 10.0;
            new
        })
        .collect();
    println!("vertices: {:#?}", plane_mesh.vertices);
    println!("indices: {:#?}", plane_mesh.indices);

    let plane_model = renderer.load_mesh(&plane_mesh, shader_program);
    entities.add_entity(vec![
        ace::Component::Model(plane_model),
        ace::Component::Position(ace::Position::default()),
    ]);
    let (width, height) = window.get_size();
    let projection = gfx::Projection {
        width: width as f32,
        height: height as f32,
        fov: 75.0,
        near: 0.1,
        far: 1000.0,
    };
    let render_system = ace::gfx::RenderSystem::new(Box::new(renderer), projection);
    let window = Arc::new(Mutex::new(window));
    //let input_system = ace::InputSystem::new(clock.clone());
    let script_system = ace::scripts::ScriptSystem;
    print_opengl_errors();
    let input_listener = ace::glfw_input::GlfwInputListener::init(window.clone());
    let mut world = ace::World::init(
        entities,
        vec![
            /*Box::new(input_system), */ Box::new(script_system),
            Box::new(render_system),
        ],
        clock.clone(),
        Box::new(input_listener),
    );
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

fn load_image(path: &Path) -> gfx::Image {
    let texture = image::ImageReader::open(path)
        .expect("Failed loading texture")
        .decode()
        .unwrap();
    let texture = texture.into_rgba8();
    gfx::Image {
        data: texture.pixels().flat_map(|p| p.0).collect(),
        width: texture.width(),
        height: texture.height(),
    }
}

fn create_point_light(
    renderer: &mut gfx::opengl::OpenGlRenderer,
    mesh: &gfx::Mesh,
    shader: u32,
) -> gfx::PointLight {
    let light_color = vec3!(1.0, 1.0, 1.0);
    let model = renderer.load_mesh(mesh, shader);
    let color = gfx::Material {
        ambient: &vec3!(0.5, 0.5, 0.5) * &light_color,
        diffuse: &vec3!(0.2, 0.2, 0.2) * &light_color,
        specular: &vec3!(1.0, 1.0, 1.0) * &light_color,
    };
    gfx::PointLight {
        model,
        color,
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
    }
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

fn create_spotlight(shader_program: u32) -> gfx::Light {
    let spotlight_color = vec3!(1.0, 1.0, 1.0);
    //let spotlight_color = vec3!(1.0, 0.0, 0.0);
    let spot_light = gfx::SpotLight {
        shader: shader_program,
        direction: Default::default(),
        position: Default::default(),
        inner_cutoff: math::radians(10.0).cos(),
        outer_cutoff: math::radians(15.0).cos(),
        material: gfx::Material {
            ambient: &vec3!(0.2, 0.2, 0.2) * &spotlight_color,
            diffuse: &vec3!(0.5, 0.5, 0.5) * &spotlight_color,
            specular: &vec3!(1.0, 1.0, 1.0) * &spotlight_color,
        },
    };
    gfx::Light::Spot(spot_light)
}
