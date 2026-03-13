use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ptr::{null, null_mut},
};

use crate::{gfx::*, vec4};

pub struct OpenGlRenderer {
    textures: u32,
}

impl OpenGlRenderer {
    pub fn init() -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
        }
        Self { textures: 0 }
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh, shader: Shader) -> Model {
        let diffuse = self.new_tex();
        self.set_texture(diffuse, &mesh.diffuse);
        let specular = self.new_tex();
        self.set_texture(specular, &mesh.specular);
        let emission = self.new_tex();
        self.set_texture(emission, &mesh.emission);
        let vao = self.set_mesh_vao(mesh);
        Model {
            vao,
            indices: mesh.indices.len() as i32,
            shader,
            material: Texture {
                diffuse: diffuse as i32,
                specular: specular as i32,
                emission: emission as i32,
                shininess: mesh.shininess,
            },
            transform: Transform::default(),
            vertices: mesh.vertices.len() as i32,
        }
    }

    fn new_tex(&mut self) -> u32 {
        self.textures += 1;
        self.textures
    }

    fn set_mesh_vao(&self, mesh: &Mesh) -> u32 {
        let mut vertex_array = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = (size_of::<Vertex>() * mesh.vertices.len()) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                mesh.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            if !mesh.indices.is_empty() {
                let mut ebo = 0u32;
                gl::GenBuffers(1, &mut ebo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                let indices_size = (size_of::<u32>() * mesh.indices.len()) as isize;
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    indices_size,
                    mesh.indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            }
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, position) as *const _,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, texture) as *const _,
            );
            gl::EnableVertexAttribArray(2);
        };
        vertex_array
    }

    fn set_texture(&self, unit: u32, image: &Image) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                image.width as i32,
                image.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    pub fn compile_shader(
        &self,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader, OpenGlError> {
        unsafe {
            let vertex_shader = Self::compile(vertex_shader, gl::VERTEX_SHADER)?;
            let fragment_shader =
                Self::compile(fragment_shader, gl::FRAGMENT_SHADER).inspect_err(|_| {
                    gl::DeleteShader(vertex_shader);
                })?;
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            let mut success = 0;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetProgramInfoLog(shader_program, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                gl::DeleteShader(fragment_shader);
                gl::DeleteShader(vertex_shader);
                return Err(OpenGlError { err });
            }
            gl::DeleteShader(fragment_shader);
            gl::DeleteShader(vertex_shader);
            Ok(shader_program)
        }
    }

    /// [Shader] needs to be freed using [gl::DeleteShader()]
    unsafe fn compile(
        shader_source: &str,
        shader_type: gl::types::GLenum,
    ) -> Result<gl::types::GLuint, OpenGlError> {
        unsafe {
            let shader_source =
                CString::new(shader_source).expect("Could not convert shader source to CString");
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(shader, 1, &shader_source.as_ptr(), null());
            gl::CompileShader(shader);
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetShaderInfoLog(shader, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                gl::DeleteShader(shader);
                return Err(OpenGlError { err });
            }
            Ok(shader)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpenGlError {
    err: String,
}
impl Error for OpenGlError {}
impl Display for OpenGlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl Renderer for OpenGlRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, models: &[Model], lights: &[Light]) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };
        let projection = projection.to_projection_matrix();
        let view = camera.to_view_matrix();
        for model in models {
            set_model_uniforms(&projection, &view, model);
            let mut dir_count = 0;
            let mut point_count = 0;
            let mut spot_count = 0;
            for light in lights {
                match light {
                    Light::Directional(l) => {
                        gl_dirlight_uniform(dir_count, model.shader, l);
                        dir_count += 1;
                    }
                    Light::Point(l) => {
                        gl_pointlight_uniform(point_count, model.shader, l, &view);
                        point_count += 1;
                    }
                    Light::Spot(l) => {
                        gl_spotlight_uniform(spot_count, model.shader, l, &view);
                        spot_count += 1;
                    }
                }
            }
            gl_int_uniform(model.shader, dir_count, "uDirectionalLightsSize");
            gl_int_uniform(model.shader, point_count, "uPointLightsSize");
            gl_int_uniform(model.shader, spot_count, "uSpotLightsSize");
            render(model);
        }
        for light in lights {
            if let Light::Point(light) = light {
                let model = &light.model;
                set_model_uniforms(&projection, &view, model);
                render(model);
            }
        }
    }
}

fn set_model_uniforms(projection: &math::Matrix4, view: &math::Matrix4, model: &Model) {
    gl_bind_vao(model.vao);
    gl_use_program(model.shader);
    gl_matrix_uniform(model.shader, projection, "uProjection");
    gl_matrix_uniform(model.shader, view, "uView");
    let model_matrix = calculate_model_matrix(&model.transform);
    gl_matrix_uniform(model.shader, &model_matrix, "uModel");
    let normal = model_matrix.inverse().transpose();
    gl_matrix_uniform(model.shader, &normal, "uNormal");
    gl_material_uniform(model.shader, &model.material);
}

fn gl_use_program(shader: Shader) {
    unsafe {
        gl::UseProgram(shader);
    }
}

fn gl_matrix_uniform(shader: Shader, matrix: &math::Matrix4, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::UniformMatrix4fv(location, 1, gl::TRUE, matrix.data.as_ptr() as *const _) }
}

fn calculate_model_matrix(transform: &Transform) -> math::Matrix4 {
    let translation = math::Matrix4::translation(&transform.position);
    let rotation = math::rotation(&transform.rotation);
    translation * rotation
}

fn gl_material_uniform(shader: Shader, texture: &Texture) {
    gl_int_uniform(shader, texture.diffuse, "uMaterial.diffuse");
    gl_int_uniform(shader, texture.specular, "uMaterial.specular");
    gl_int_uniform(shader, texture.emission, "uMaterial.emission");
    gl_float_uniform(shader, texture.shininess, "uMaterial.shininess");
}

fn gl_int_uniform(shader: Shader, value: i32, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform1i(location, value) }
}

fn gl_get_uniform_location(shader: Shader, key: &str) -> gl::types::GLint {
    let key = CString::new(key).unwrap();
    unsafe { gl::GetUniformLocation(shader, key.as_ptr()) }
}

fn gl_float_uniform(shader: Shader, value: f32, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform1f(location, value) }
}

fn gl_dirlight_uniform(key: i32, shader: Shader, light: &DirectionalLight) {
    let key = &format!("uDirectionalLights[{key}]");
    gl_vec3_uniform(shader, &light.direction, &subkey(key, "direction"));
    gl_light_uniform(shader, &light.material, &subkey(key, "light"));
}
fn to_view_space(view: &math::Matrix4, vec: &math::Vec3, w: f32) -> math::Vec3 {
    let res = view * &vec4!(vec.x, vec.y, vec.z, w);
    vec3!(res.x, res.y, res.z)
}

fn gl_light_uniform(shader: Shader, light: &Material, key: &str) {
    gl_vec3_uniform(shader, &light.ambient, &subkey(key, "ambient"));
    gl_vec3_uniform(shader, &light.diffuse, &subkey(key, "diffuse"));
    gl_vec3_uniform(shader, &light.specular, &subkey(key, "specular"));
}

fn subkey(key: &str, subkey: &str) -> String {
    format!("{key}.{subkey}")
}

fn gl_vec3_uniform(shader: Shader, value: &math::Vec3, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform3f(location, value.x, value.y, value.z) }
}

fn gl_pointlight_uniform(key: i32, shader: Shader, light: &PointLight, view: &math::Matrix4) {
    let key = &format!("uPointLights[{key}]");
    let position = to_view_space(view, &light.model.transform.position, 1.0);
    gl_float_uniform(shader, light.constant, &subkey(key, "constant"));
    gl_float_uniform(shader, light.linear, &subkey(key, "linear"));
    gl_float_uniform(shader, light.quadratic, &subkey(key, "quadratic"));
    gl_vec3_uniform(shader, &position, &subkey(key, "position"));
    gl_light_uniform(shader, &light.color, &subkey(key, "light"));
}

fn gl_spotlight_uniform(key: i32, shader: Shader, light: &SpotLight, view: &math::Matrix4) {
    let key = &format!("uSpotLights[{key}]");
    let position = to_view_space(view, &light.position, 1.0);
    let direction = to_view_space(view, &light.direction, 0.0);
    gl_vec3_uniform(shader, &position, &subkey(key, "position"));
    gl_vec3_uniform(shader, &direction, &subkey(key, "direction"));
    gl_float_uniform(shader, light.inner_cutoff, &subkey(key, "cutoff"));
    gl_float_uniform(shader, light.outer_cutoff, &subkey(key, "outerCutoff"));
    gl_light_uniform(shader, &light.material, &subkey(key, "light"));
}

fn render(model: &Model) {
    if model.indices > 0 {
        gl_draw_elements(model.indices);
    } else {
        gl_draw_array(model.vertices);
    }
}

fn gl_draw_array(vertices: i32) {
    unsafe { gl::DrawArrays(gl::TRIANGLES, 0, vertices) }
}

fn gl_draw_elements(indices: i32) {
    unsafe { gl::DrawElements(gl::TRIANGLES, indices, gl::UNSIGNED_INT, null()) }
}

fn gl_bind_vao(vao: VertexArray) {
    unsafe { gl::BindVertexArray(vao) }
}
