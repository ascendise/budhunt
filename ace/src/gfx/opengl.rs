use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ptr::{null, null_mut},
};

use crate::{gfx::*, vec4};

pub struct OpenGlRenderer {
    texture_count: u32,
    skybox: Option<Skybox>,
}
impl Renderer for OpenGlRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, models: &[Model], lights: &[Light]) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };
        let projection = &projection.to_projection_matrix();
        let view = &camera.to_view_matrix();
        let skybox = self.skybox.as_ref().expect("No skybox set!");
        let shader = ModelShader {
            projection,
            view,
            models,
            lights,
            skybox,
        };
        shader.render();
        self.render_skybox(projection, view);
    }
}
impl OpenGlRenderer {
    pub fn init() -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }
        Self {
            texture_count: 0,
            skybox: None,
        }
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh, shader: Shader) -> Model {
        let nodes = mesh
            .nodes
            .iter()
            .map(|n| self.load_mesh_node(n, shader))
            .collect();
        Model { nodes }
    }

    fn load_mesh_node(&mut self, node: &MeshNode, shader: Shader) -> ModelNode {
        let albedo = self.new_tex();
        self.set_texture(albedo, &node.albedo, gl::SRGB_ALPHA as i32, gl::RGBA);
        let metallic_roughness_ao = self.new_tex();
        self.set_texture(
            metallic_roughness_ao,
            &node.metallic_roughness_ao,
            gl::RGB8 as i32,
            gl::RGB,
        );
        let vao = self.set_mesh_vao(node);
        ModelNode {
            vao,
            indices: node.indices.len() as i32,
            shader,
            material: Texture {
                albedo: albedo as i32,
                metallic_roughness_ao: metallic_roughness_ao as i32,
            },
            transform: Transform::default(),
            vertices: node.vertices.len() as i32,
        }
    }

    fn new_tex(&mut self) -> u32 {
        self.texture_count += 1;
        self.texture_count
    }

    fn set_mesh_vao(&self, mesh: &MeshNode) -> u32 {
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

    fn set_texture(
        &self,
        unit: u32,
        image: &Image,
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    fn set_texture_f32(
        &self,
        unit: u32,
        image: &Image<f32>,
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::FLOAT,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    pub fn compile_shader(
        &self,
        vertex_shader: &str,
        fragment_shader: &str,
        tonemap_shader: &str,
    ) -> Result<Shader, OpenGlError> {
        unsafe {
            let vertex_shader = Self::compile(vertex_shader, gl::VERTEX_SHADER)?;
            let fragment_shader =
                Self::compile(fragment_shader, gl::FRAGMENT_SHADER).inspect_err(|_| {
                    gl::DeleteShader(vertex_shader);
                })?;
            let tonemap_shader =
                Self::compile(tonemap_shader, gl::FRAGMENT_SHADER).inspect_err(|_| {
                    gl::DeleteShader(vertex_shader);
                    gl::DeleteShader(fragment_shader);
                })?;
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::AttachShader(shader_program, tonemap_shader);
            gl::LinkProgram(shader_program);
            let mut success = 0;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetProgramInfoLog(shader_program, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                gl::DeleteShader(tonemap_shader);
                gl::DeleteShader(fragment_shader);
                gl::DeleteShader(vertex_shader);
                return Err(OpenGlError { err });
            }
            gl::DeleteShader(tonemap_shader);
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

    pub fn set_skybox(&mut self, skybox: &gfx::Ibl, shader: Shader) {
        let image = self.new_tex();
        self.set_texture_f32(image, &skybox.skybox, gl::RGB32F as i32, gl::RGB);
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        let diffuse = self.new_tex();
        self.set_texture_f32(diffuse, &skybox.diffuse, gl::RGB32F as i32, gl::RGB);
        let specular = self.new_tex();
        self.set_texture_with_mip_levels(specular, &skybox.specular, gl::RGB32F as i32, gl::RGB);
        let brdf_lut = self.new_tex();
        self.set_texture(brdf_lut, &skybox.brdf_lut, gl::RGB8 as i32, gl::RGB);
        let mut vertex_array = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = (size_of::<math::Vec3>() * Skybox::VERTICES.len()) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                Skybox::VERTICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<math::Vec3>() as i32,
                null() as *const _,
            );
            gl::EnableVertexAttribArray(0);
        };
        let skybox = Skybox {
            vao: vertex_array,
            shader,
            image: image as i32,
            specular: specular as i32,
            diffuse: diffuse as i32,
            brdf_lut: brdf_lut as i32,
        };
        self.skybox = Some(skybox);
    }

    fn set_texture_with_mip_levels(
        &self,
        unit: u32,
        images: &[gfx::Image<f32>],
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            for (i, image) in images.iter().enumerate() {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    i as i32,
                    internal_format,
                    image.width as i32,
                    image.height as i32,
                    0,
                    format,
                    gl::FLOAT,
                    image.data.as_ptr() as *const _,
                );
            }
        }
    }

    fn render_skybox(&self, projection: &math::Matrix4, view: &math::Matrix4) {
        let view: math::Matrix4 = [
            [view[0][0], view[0][1], view[0][2], 0.0],
            [view[1][0], view[1][1], view[1][2], 0.0],
            [view[2][0], view[2][1], view[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into();
        if let Some(skybox) = &self.skybox {
            let skybox_shader = SkyboxShader {
                view: &view,
                projection,
                skybox,
            };
            skybox_shader.render();
        }
    }
}

pub trait OpenGlShader {
    fn render(&self);
}
pub struct SkyboxShader<'a> {
    view: &'a math::Matrix4,
    projection: &'a math::Matrix4,
    skybox: &'a Skybox,
}
impl<'a> OpenGlShader for SkyboxShader<'a> {
    fn render(&self) {
        unsafe {
            let shader = self.skybox.shader;
            gl::BindVertexArray(self.skybox.vao);
            gl::UseProgram(shader);
            gl_int_uniform(shader, self.skybox.image, "uSkybox");
            gl_matrix_uniform(shader, self.view, "uView");
            gl_matrix_uniform(shader, self.projection, "uProjection");
            gl_float_uniform(shader, 2.2, "uGamma");
            gl_float_uniform(shader, 0.1, "uExposure");
            gl::DepthFunc(gl::LEQUAL);
            gl::DrawArrays(gl::TRIANGLES, 0, Skybox::VERTICES.len() as i32);
            gl::DepthFunc(gl::LESS);
        }
    }
}
pub struct ModelShader<'a> {
    projection: &'a math::Matrix4,
    view: &'a math::Matrix4,
    models: &'a [Model],
    lights: &'a [Light],
    skybox: &'a Skybox,
}
impl<'a> OpenGlShader for ModelShader<'a> {
    fn render(&self) {
        unsafe {
            for model in self.models.iter().flat_map(|m| &m.nodes) {
                gl::BindVertexArray(model.vao);
                gl::UseProgram(model.shader);
                self.set_model_uniforms(model);
                self.set_skybox_uniforms(model);
                let mut point_count = 0;
                for light in self.lights {
                    match light {
                        Light::Point(light) => {
                            let key = &format!("uPointLights[{point_count}]");
                            gl_vec3_uniform(model.shader, &light.color, &subkey(key, "color"));
                            gl_vec3_uniform(
                                model.shader,
                                &to_view_space(self.view, &light.position, 1.0),
                                &subkey(key, "position"),
                            );
                            point_count += 1;
                        }
                    }
                }
                gl_int_uniform(model.shader, point_count, "uPointLightsSize");
                if model.indices > 0 {
                    gl::DrawElements(gl::TRIANGLES, model.indices, gl::UNSIGNED_INT, null());
                } else {
                    gl::DrawArrays(gl::TRIANGLES, 0, model.vertices);
                }
            }
        }
    }
}
impl<'a> ModelShader<'a> {
    fn create_model_matrix(transform: &Transform) -> math::Matrix4 {
        let translation = math::Matrix4::translation(&transform.position);
        let rotation = math::rotation(&transform.rotation);
        translation * rotation
    }

    fn create_normal_matrix(model: &math::Matrix4, view: &math::Matrix4) -> math::Matrix4 {
        (model * view).inverse().transpose()
    }

    fn set_model_uniforms(&self, model: &ModelNode) {
        gl_matrix_uniform(model.shader, self.projection, "uProjection");
        gl_matrix_uniform(model.shader, self.view, "uView");
        let model_matrix = Self::create_model_matrix(&model.transform);
        gl_matrix_uniform(model.shader, &model_matrix, "uModel");
        let normal_matrix = Self::create_normal_matrix(&model_matrix, self.view);
        gl_matrix_uniform(model.shader, &normal_matrix, "uNormal");
        gl_int_uniform(model.shader, model.material.albedo, "uMaterial.albedo");
        gl_int_uniform(
            model.shader,
            model.material.metallic_roughness_ao,
            "uMaterial.metallicRoughnessAo",
        );
        gl_float_uniform(model.shader, 0.5, "uExposure");
    }

    fn set_skybox_uniforms(&self, model: &ModelNode) {
        gl_int_uniform(model.shader, self.skybox.diffuse, "uIrradianceMap");
        gl_int_uniform(model.shader, self.skybox.specular, "uPrefilterMap");
        gl_int_uniform(model.shader, self.skybox.brdf_lut, "uBrdfLut");
    }
}

fn gl_matrix_uniform(shader: Shader, matrix: &math::Matrix4, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::UniformMatrix4fv(location, 1, gl::TRUE, matrix.data.as_ptr() as *const _) }
}

fn gl_vec3_uniform(shader: Shader, value: &math::Vec3, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform3f(location, value.x, value.y, value.z) }
}

fn gl_int_uniform(shader: Shader, value: i32, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform1i(location, value) }
}

fn gl_float_uniform(shader: Shader, value: f32, key: &str) {
    let location = gl_get_uniform_location(shader, key);
    unsafe { gl::Uniform1f(location, value) }
}

fn gl_get_uniform_location(shader: Shader, key: &str) -> gl::types::GLint {
    let key = CString::new(key).unwrap();
    unsafe { gl::GetUniformLocation(shader, key.as_ptr()) }
}

fn subkey(key: &str, subkey: &str) -> String {
    format!("{key}.{subkey}")
}

fn to_view_space(view: &math::Matrix4, vec: &math::Vec3, w: f32) -> math::Vec3 {
    let res = view * &vec4!(vec.x, vec.y, vec.z, w);
    vec3!(res.x, res.y, res.z)
}

#[derive(Debug, Clone)]
pub struct Skybox {
    vao: VertexArray,
    shader: Shader,
    image: Tex,
    diffuse: Tex,
    specular: Tex,
    brdf_lut: Tex,
}
impl Skybox {
    pub const VERTICES: [math::Vec3; 36] = [
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, -1.0, -1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(-1.0, -1.0, -1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, 1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(1.0, -1.0, 1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, -1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(-1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, -1.0, 1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(1.0, 1.0, -1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, 1.0),
        vec3!(-1.0, 1.0, 1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, -1.0, -1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(1.0, -1.0, -1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(1.0, -1.0, 1.0),
    ];
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
