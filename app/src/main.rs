use glfw::Context;

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let (mut window, _) = glfw
        .create_window(1280, 720, "Budhunt", glfw::WindowMode::Windowed)
        .expect("Failed to create window");
    window.make_current();
    unsafe {
        gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);
        gl::ClearColor(0.75, 0.75, 0.75, 1.0);
        while !window.should_close() {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            window.swap_buffers();
            glfw.poll_events();
        }
    }
}
