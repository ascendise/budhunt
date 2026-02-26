use ace::{
    gfx::{self, Image, Renderer},
    math::{self, radians},
    vec3,
};
use glfw::Context;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

static VERTEX_SHADER_SOURCE: &str = include_str!("../shaders/object.vs.glsl");
static FRAGMENT_SHADER_SOURCE: &str = include_str!("../shaders/object.fs.glsl");

fn main() {
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
    let point_lights = [
        vec3!(0.7, 0.2, 2.0),
        vec3!(2.3, -3.3, -4.0),
        vec3!(-4.0, 2.0, -12.0),
        vec3!(0.0, 0.0, -3.0),
    ];

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let mut window = setup_window(&mut glfw);
    let mut renderer = gfx::opengl::OpenGlRenderer::init();
    let shader_program = renderer
        .compile_shader(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
        .expect("Failed to compile model shader");
    let specular_map = load_image(Path::new(
        "/home/ascendise/dev/budhunt/app/models/suzanne_specular.png",
    ));
    let monkey_mesh = gfx::load_glb_file(
        Path::new("/home/ascendise/dev/budhunt/app/models/Suzanne.glb"),
        &specular_map,
    );
    let monkey_model = renderer.load_mesh(&monkey_mesh, shader_program);
    let light_mesh = gfx::load_glb_file(
        Path::new("/home/ascendise/dev/budhunt/app/models/Light.glb"),
        &Image::empty(),
    );
    let point_lights = load_point_lights(&mut renderer, &light_mesh, &point_lights, shader_program);
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
    let dir_light = gfx::Light::Directional(dir_light);
    print_opengl_errors();
    let mut last_frame = 0f32;
    let mut eye = vec3!(0.0, 0.0, 3.0);
    while !window.glfw.should_close() {
        handle_key_events(&mut window, &renderer);
        let fov = radians(*window.fov.lock().unwrap());
        let projection = gfx::Projection {
            width: window.width as f32,
            height: window.height as f32,
            fov,
            near: 0.1,
            far: 100.0,
        };
        let (move_direction, camera_direction) =
            get_camera_direction(&window.cursor_offset.lock().unwrap());
        let time = glfw.get_time() as f32;
        let delta_time = time_delta(time, &mut last_frame);
        eye = move_camera(&window.glfw, &eye, &move_direction, 10.0, delta_time);
        let camera = gfx::Camera {
            position: eye.clone(),
            direction: camera_direction.clone(),
        };
        let models = load_models(&monkeys, &monkey_model, time);
        let lights = setup_lightning(shader_program, &point_lights, &dir_light, &eye, &camera);
        renderer.render(&projection, &camera, &models, &lights);
        window.glfw.swap_buffers();
        glfw.poll_events();
    }
}

fn setup_window(glfw: &mut glfw::Glfw) -> Window {
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
    let (width, height) = window.get_size();
    let cursor_position = Arc::new(Mutex::new(vec3!(width as f32, height as f32, 0.0)));
    let cursor_position_2 = cursor_position.clone();
    let cursor_offset = Arc::new(Mutex::new(math::Vec3::default()));
    let cursor_offset_2 = cursor_offset.clone();
    window.set_cursor_pos_callback(move |_, x, y| {
        let sensitivity = 0.1;
        let mut cursor_offset = cursor_offset_2.lock().unwrap();
        let mut cursor_position = cursor_position_2.lock().unwrap();
        let x = x as f32;
        let y = y as f32;
        let offset_x = x - cursor_position.x;
        let offset_y = cursor_position.y - y;
        cursor_position.x = x;
        cursor_position.y = y;
        cursor_offset.x += offset_x * sensitivity;
        cursor_offset.y = (cursor_offset.y + offset_y * sensitivity).clamp(-89.0, 89.0);
    });
    let fov = Arc::new(Mutex::new(45.032));
    let fov_write = fov.clone();
    window.set_scroll_callback(move |_, _, y| {
        let speed = 10.0;
        let y = y as f32;
        let mut fov = fov_write.lock().unwrap();
        *fov = (*fov - (y * speed)).clamp(1.0, 90.0);
    });
    window.make_current();
    Window {
        glfw: window,
        width,
        height,
        cursor_offset,
        fov,
    }
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

fn time_delta(now: f32, last_frame: &mut f32) -> f32 {
    let delta_time = now - *last_frame;
    *last_frame = now;
    delta_time
}

fn handle_key_events(window: &mut Window, renderer: &gfx::opengl::OpenGlRenderer) {
    if window.glfw.get_key(glfw::Key::Space) == glfw::Action::Press {
        renderer.set_polygon_mode(gl::LINE);
    }
    if window.glfw.get_key(glfw::Key::Space) == glfw::Action::Release {
        renderer.set_polygon_mode(gl::FILL);
    }
    if window.glfw.get_key(glfw::Key::Escape) == glfw::Action::Press {
        window.glfw.set_should_close(true);
    }
}

fn load_point_lights(
    renderer: &mut gfx::opengl::OpenGlRenderer,
    mesh: &gfx::Mesh,
    positions: &[math::Vec3],
    shader: u32,
) -> Vec<gfx::Light> {
    let light_color = vec3!(1.0, 1.0, 1.0);
    let model = renderer.load_mesh(mesh, shader);
    let color = gfx::Material {
        ambient: &vec3!(0.5, 0.5, 0.5) * &light_color,
        diffuse: &vec3!(0.2, 0.2, 0.2) * &light_color,
        specular: &vec3!(1.0, 1.0, 1.0) * &light_color,
    };
    let point_light = gfx::PointLight {
        model,
        color,
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
    };
    let point_lights: Vec<gfx::Light> = positions
        .iter()
        .map(|p| {
            let mut point_light = point_light.clone();
            point_light.model.transform.position = p.clone();
            gfx::Light::Point(point_light)
        })
        .collect();
    point_lights
}

pub struct Window {
    glfw: glfw::PWindow,
    width: i32,
    height: i32,
    cursor_offset: Arc<Mutex<math::Vec3>>,
    fov: Arc<Mutex<f32>>,
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

/// Returns (movement, camera) direction
fn get_camera_direction(cursor_offset: &math::Vec3) -> (math::Vec3, math::Vec3) {
    let yaw = radians(cursor_offset.x);
    let pitch = radians(cursor_offset.y);
    let movement = math::Vec3 {
        x: yaw.cos(),
        y: 0.0,
        z: yaw.sin(),
    }
    .normalize();
    let direction = math::Vec3 {
        x: yaw.cos() * pitch.cos(),
        y: pitch.sin(),
        z: yaw.sin() * pitch.cos(),
    }
    .normalize();
    (movement, direction)
}

fn move_camera(
    window: &glfw::Window,
    eye: &math::Vec3,
    direction: &math::Vec3,
    speed: f32,
    delta_time: f32,
) -> math::Vec3 {
    let mut movement = math::Vec3::default();
    let speed = delta_time * speed;
    let front = direction * speed;
    let up = vec3!(0.0, 1.0, 0.0);
    let strafe = front.cross(&up).normalize() * speed;
    if window.get_key(glfw::Key::W) == glfw::Action::Press {
        movement = &movement + &front;
    }
    if window.get_key(glfw::Key::S) == glfw::Action::Press {
        movement = &movement - &front;
    }
    if window.get_key(glfw::Key::D) == glfw::Action::Press {
        movement = &(&movement / 2.0) + &strafe;
    }
    if window.get_key(glfw::Key::A) == glfw::Action::Press {
        movement = &(&movement / 2.0) - &strafe;
    }
    eye + &movement
}

fn load_models(positions: &[math::Vec3; 10], model: &gfx::Model, time: f32) -> Vec<gfx::Model> {
    let models: Vec<gfx::Model> = positions
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let angle = time * f32::max(i as f32, 0.5);
            let rotation = vec3!(radians(1.0 * angle), radians(1.0 * angle), 0.0);
            let mut model = model.clone();
            model.transform = gfx::Transform {
                position: c.clone(),
                rotation,
            };
            model
        })
        .collect();
    models
}

fn setup_lightning(
    shader_program: u32,
    point_lights: &[gfx::Light],
    dir_light: &gfx::Light,
    eye: &math::Vec3,
    camera: &gfx::Camera,
) -> Vec<gfx::Light> {
    let spotlight_color = vec3!(1.0, 1.0, 1.0);
    let spot_light = gfx::SpotLight {
        shader: shader_program,
        direction: camera.direction.clone(),
        position: eye.clone(),
        inner_cutoff: math::radians(10.0).cos(),
        outer_cutoff: math::radians(15.0).cos(),
        material: gfx::Material {
            ambient: &vec3!(0.2, 0.2, 0.2) * &spotlight_color,
            diffuse: &vec3!(0.5, 0.5, 0.5) * &spotlight_color,
            specular: &vec3!(1.0, 1.0, 1.0) * &spotlight_color,
        },
    };
    let spot_light = gfx::Light::Spot(spot_light);
    let mut lights = point_lights.to_vec();
    lights.push(dir_light.clone());
    lights.push(spot_light.clone());
    lights
}
