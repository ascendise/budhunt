use indexmap::indexmap;
use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ptr::{null, null_mut},
};

use crate::gfx::*;

macro_rules! assert_no_gl_error {
    // Is a macro so printed stack trace starts at the line the assert failed instead of at the assert function
    ($msg:literal) => {
        unsafe {
            let res = gl::GetError();
            if res != 0 {
                panic!("{} {res}", $msg)
            }
        }
    };
}

pub struct OpenGlRenderer {
    state: State,
    skybox: Option<Skybox>,
    line_vao: Idx,
}
impl Renderer for OpenGlRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, renderables: &[Renderable]) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };
        let projection = &projection.to_projection_matrix();
        let view = &camera.to_view_matrix();
        let skybox = self.skybox.as_ref().expect("No skybox set!");
        let models: Vec<Model> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Model))
            .cloned()
            .collect();
        let lights: Vec<Light> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Light))
            .cloned()
            .collect();
        let shader = ModelShader {
            state: &self.state,
            projection,
            view,
            models: &models,
            lights: &lights,
            skybox,
        };
        shader.render();
        self.render_skybox(projection, view);
        self.render_lines(projection, view, renderables);
    }
}
impl OpenGlRenderer {
    pub const LINE: [math::Vec3; 2] = [vec3!(0.0), vec3!(0.0, 0.0, -100.0)];

    pub fn init() -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            let mut state = State::new();
            let line_vao = Self::init_line(&mut state);
            Self {
                state,
                skybox: None,
                line_vao,
            }
        }
    }

    fn init_line(state: &mut State) -> Idx {
        let vao = VertexArray::simple_new(&Self::LINE);
        state.vertex_arrays.insert(vao)
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh, program: Idx) -> ModelNodes {
        mesh.nodes
            .iter()
            .map(|n| self.load_mesh_node(n, program))
            .collect()
    }

    fn load_mesh_node(&mut self, node: &MeshNode, program: Idx) -> ModelNode {
        let albedo = self.state.textures.insert(Texture::new(
            &node.albedo,
            gl::UNSIGNED_BYTE,
            &TextureFormat::SRGB_ALPHA,
            true,
        ));
        let metallic_roughness_ao = self.state.textures.insert(Texture::new(
            &node.metallic_roughness_ao,
            gl::UNSIGNED_BYTE,
            &TextureFormat::RGB,
            true,
        ));
        let vao = self.set_mesh_vao(node);
        ModelNode {
            vao,
            indices: node.indices.len() as i32,
            program,
            material: super::Texture {
                albedo,
                metallic_roughness_ao,
            },
            vertices: node.vertices.len() as i32,
        }
    }

    fn set_mesh_vao(&mut self, mesh: &MeshNode) -> usize {
        //TODO: make mesh.indices an Option<>
        let indices = if mesh.indices.is_empty() {
            None
        } else {
            Some(&mesh.indices as &[u32])
        };
        let vao = VertexArray::new(&mesh.vertices, indices);
        self.state.vertex_arrays.insert(vao)
    }

    pub fn compile_shader(
        &mut self,
        vertex_shader: &str,
        fragment_shader: &str,
        tonemap_shader: &str,
    ) -> Result<Idx, OpenGlError> {
        unsafe {
            let vertex_shader = Shader::compile(vertex_shader, gl::VERTEX_SHADER)?;
            let fragment_shader = Shader::compile(fragment_shader, gl::FRAGMENT_SHADER)?;
            let tonemap_shader = Shader::compile(tonemap_shader, gl::FRAGMENT_SHADER)?;
            let program = Program::link(&[vertex_shader, fragment_shader, tonemap_shader])?;
            let program = self.state.programs.insert(program);
            Ok(program)
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

    pub fn set_skybox(&mut self, skybox: &gfx::Ibl, shader: Idx) {
        let image = Texture::new(&skybox.skybox, gl::FLOAT, &TextureFormat::RGB32F, true);
        image.texture_filter(gl::LINEAR as i32, gl::LINEAR as i32);
        let image = self.state.textures.insert(image);
        let diffuse = Texture::new(&skybox.diffuse, gl::FLOAT, &TextureFormat::RGB32F, true);
        let diffuse = self.state.textures.insert(diffuse);
        let specular =
            Texture::new_with_levels(&skybox.specular, gl::FLOAT, &TextureFormat::RGB32F);
        specular.texture_filter(gl::LINEAR_MIPMAP_LINEAR as i32, gl::LINEAR as i32);
        let specular = self.state.textures.insert(specular);
        let brdf_lut = Texture::new(
            &skybox.brdf_lut,
            gl::UNSIGNED_BYTE,
            &TextureFormat::RGB,
            true,
        );
        let brdf_lut = self.state.textures.insert(brdf_lut);
        let vao = VertexArray::simple_new(&Skybox::VERTICES);
        let vao = self.state.vertex_arrays.insert(vao);
        let skybox = Skybox {
            vao,
            program: shader,
            image,
            specular,
            diffuse,
            brdf_lut,
        };
        self.skybox = Some(skybox);
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
                state: &self.state,
                view: &view,
                projection,
                skybox,
            };
            skybox_shader.render();
        }
    }

    fn render_lines(
        &self,
        projection: &math::Matrix4,
        view: &math::Matrix4,
        renderables: &[Renderable],
    ) {
        let lines: Vec<Line> = renderables
            .iter()
            .filter_map(|r| maybe_component!(r, Renderable::Line))
            .cloned()
            .collect();
        let shader = LineShader {
            state: &self.state,
            vao: &self.state.vertex_arrays[self.line_vao],
            lines: &lines,
            projection,
            view,
        };
        shader.render();
    }
}

pub type Idx = usize;
#[derive(Debug, PartialEq, Eq)]
struct State {
    vertex_arrays: StateCollection<VertexArray>,
    textures: StateCollection<Texture>,
    programs: StateCollection<Program>,
}
impl State {
    pub fn new() -> Self {
        Self {
            vertex_arrays: StateCollection::empty(),
            textures: StateCollection::empty(),
            programs: StateCollection::empty(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct StateCollection<T> {
    values: IndexMap<Idx, T>,
    count: Count,
}
impl<T> StateCollection<T> {
    pub fn empty() -> Self {
        Self {
            values: Default::default(),
            count: Count::init(),
        }
    }

    pub fn insert(&mut self, value: T) -> Idx {
        let key = self.count.increment();
        self.values.insert(key, value);
        key
    }
}
impl<T> Index<Idx> for StateCollection<T> {
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.values[index]
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Count(isize);
impl Count {
    pub fn init() -> Self {
        Self(-1)
    }

    /// Increments counter. First increment returns 0
    pub fn increment(&mut self) -> usize {
        self.0 += 1;
        self.0 as usize
    }

    /// returns last result of [Self::increment]
    pub fn last(&mut self) -> usize {
        if self.0 < 0 {
            panic!("counter was never incremented!");
        }
        self.0 as usize
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VertexArray(u32);
impl VertexArray {
    pub fn new(vertices: &[Vertex], indices: Option<&[u32]>) -> Self {
        unsafe {
            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let vertex_buffer = Self::create_vertex_buffer(vertices);
            let element_buffer = indices.map(Self::create_element_array_buffer);
            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &vertex_buffer as *const u32);
            if let Some(element_buffer) = element_buffer {
                gl::DeleteBuffers(1, &element_buffer as *const u32);
            }
            Self(vertex_array)
        }
    }

    fn create_vertex_buffer(vertices: &[Vertex]) -> u32 {
        unsafe {
            let mut buffer = 0;
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertices) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
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
            buffer
        }
    }

    fn create_element_array_buffer(indices: &[u32]) -> u32 {
        unsafe {
            let mut ebo = 0u32;
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let indices_size = std::mem::size_of_val(indices) as isize;
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices_size,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            ebo
        }
    }

    pub fn simple_new(vertices: &[math::Vec3]) -> Self {
        unsafe {
            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let vertex_buffer = Self::create_simple_vertex_buffer(vertices);
            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &vertex_buffer as *const u32);
            Self(vertex_array)
        }
    }

    fn create_simple_vertex_buffer(vertices: &[math::Vec3]) -> u32 {
        unsafe {
            let mut buffer = 0;
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertices) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<math::Vec3>() as i32,
                null(),
            );
            gl::EnableVertexAttribArray(0);
            buffer
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.0);
        }
    }
}
impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.0);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Texture(u32);
impl Texture {
    pub fn new<T>(
        image: &Image<T>,
        data_format: u32,
        format: &TextureFormat,
        generate_mipmap: bool,
    ) -> Self {
        unsafe {
            let texture = Self::generate_texture(std::slice::from_ref(image), data_format, format);
            if generate_mipmap {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
            Self(texture)
        }
    }

    fn generate_texture<T>(images: &[Image<T>], data_format: u32, format: &TextureFormat) -> u32 {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            for (i, image) in images.iter().enumerate() {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    i as i32,
                    format.0,
                    image.width as i32,
                    image.height as i32,
                    0,
                    format.1,
                    data_format,
                    image.data.as_ptr() as *const _,
                );
            }
            texture
        }
    }

    pub fn new_with_levels<T>(
        images: &[Image<T>],
        data_format: u32,
        format: &TextureFormat,
    ) -> Self {
        let texture = Self::generate_texture(images, data_format, format);
        Texture(texture)
    }

    /// Binds [Texture] to [texture unit](gl::TEXTURE0)
    pub fn bind(&self, unit: usize) {
        unsafe {
            let unit = gl::TEXTURE0 + unit as u32;
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, self.0);
        }
    }

    pub fn texture_filter(&self, minification: i32, magnification: i32) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, minification);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, magnification);
        }
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.0) }
    }
}
struct TextureFormat(gl::types::GLint, gl::types::GLenum);
impl TextureFormat {
    pub const SRGB_ALPHA: TextureFormat = TextureFormat(gl::SRGB_ALPHA as i32, gl::RGBA);
    pub const RGB: TextureFormat = TextureFormat(gl::RGB8 as i32, gl::RGB);
    pub const RGB32F: TextureFormat = TextureFormat(gl::RGB32F as i32, gl::RGB);
}
#[derive(Debug, PartialEq, Eq)]
struct Program(u32);
impl Program {
    pub fn link(shaders: &[Shader]) -> Result<Self, OpenGlError> {
        unsafe {
            let program = gl::CreateProgram();
            for shader in shaders {
                gl::AttachShader(program, shader.0);
            }
            gl::LinkProgram(program);
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let err = Self::get_program_info_log(program);
                gl::DeleteProgram(program);
                Err(OpenGlError { err })
            } else {
                Ok(Self(program))
            }
        }
    }

    fn get_program_info_log(program: u32) -> String {
        let mut err = ['\0' as i8; 512];
        unsafe {
            gl::GetProgramInfoLog(program, 512, null_mut(), err.as_mut_ptr());
        }
        let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
        err.trim_end_matches('\0').to_string()
    }

    pub fn use_and_set(&self, uniforms: &Uniforms) {
        unsafe {
            gl::UseProgram(self.0);
        }
        for (location, uniform) in uniforms {
            uniform.set(location);
        }
    }

    pub fn location(&self, key: &str) -> Option<Location> {
        Location::find(self, key)
    }
}
type Uniforms = IndexMap<Location, Uniform>;
enum Uniform {
    Vec3(math::Vec3),
    Mat4(math::Matrix4),
    Float(f32),
    Int(i32),
}
impl Uniform {
    pub fn set(&self, location: &Location) {
        unsafe {
            match self {
                Uniform::Vec3(vec3) => gl::Uniform3f(location.0, vec3.x(), vec3.y(), vec3.z()),
                Uniform::Mat4(mat4) => {
                    gl::UniformMatrix4fv(location.0, 1, gl::TRUE, mat4.data.as_ptr() as *const _)
                }
                Uniform::Float(float) => gl::Uniform1f(location.0, *float),
                Uniform::Int(int) => gl::Uniform1i(location.0, *int),
            }
        }
    }
}
fn subkey(key: &str, subkey: &str) -> String {
    format!("{key}.{subkey}")
}
#[derive(Eq, Hash, PartialEq)]
struct Location(gl::types::GLint);
impl Location {
    pub fn find(program: &Program, uniform_key: &str) -> Option<Self> {
        let uniform_key = CString::new(uniform_key).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(program.0, uniform_key.as_ptr());
            if location == -1 {
                return None;
            }
            let location = Self(location);
            Some(location)
        }
    }
}
struct Shader(gl::types::GLuint);
impl Shader {
    pub fn compile(source: &str, shader_type: gl::types::GLenum) -> Result<Self, OpenGlError> {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            let source = CString::new(source).expect("Could not convert shader source to CString");
            gl::ShaderSource(shader, 1, &source.as_ptr(), null());
            gl::CompileShader(shader);
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let err = Self::get_shader_info_log(shader);
                gl::DeleteShader(shader);
                return Err(OpenGlError { err });
            }
            Ok(Self(shader))
        }
    }

    fn get_shader_info_log(shader: u32) -> String {
        unsafe {
            let mut err = ['\0' as i8; 512];
            gl::GetShaderInfoLog(shader, 512, null_mut(), err.as_mut_ptr());
            let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
            err.trim_end_matches('\0').to_string()
        }
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}

trait OpenGlShader {
    fn render(&self);
}

struct ModelShader<'a> {
    state: &'a State,
    projection: &'a math::Matrix4,
    view: &'a math::Matrix4,
    models: &'a [Model],
    lights: &'a [Light],
    skybox: &'a Skybox,
}
impl<'a> OpenGlShader for ModelShader<'a> {
    fn render(&self) {
        unsafe {
            for model in self.models {
                for node in &model.nodes {
                    let mut texture_count = Count::init();
                    self.state.vertex_arrays[node.vao].bind();
                    let program = &self.state.programs[node.program];
                    let uniforms =
                        self.get_model_shader_uniforms(program, node, model, texture_count);
                    program.use_and_set(&uniforms);
                    if node.indices > 0 {
                        gl::DrawElements(gl::TRIANGLES, node.indices, gl::UNSIGNED_INT, null());
                    } else {
                        gl::DrawArrays(gl::TRIANGLES, 0, node.vertices);
                    }
                }
            }
        }
    }
}
impl<'a> ModelShader<'a> {
    fn create_normal_matrix(model: &math::Matrix4, view: &math::Matrix4) -> math::Matrix4 {
        (model * view).inverse().transpose()
    }

    fn get_model_shader_uniforms(
        &self,
        program: &Program,
        node: &ModelNode,
        model: &Model,
        mut texture_count: Count,
    ) -> IndexMap<Location, Uniform> {
        let mut uniforms = IndexMap::new();
        self.add_model_uniforms(
            program,
            &mut uniforms,
            node,
            &model.transform,
            &mut texture_count,
        );
        self.add_skybox_uniforms(program, &mut uniforms, node, &mut texture_count);
        let mut point_count = 0;
        for light in self.lights {
            match light {
                Light::Point(light) => {
                    let key = &format!("uPointLights[{point_count}]");
                    uniforms.insert(
                        program.location(&format!("{key}.color")).unwrap(),
                        Uniform::Vec3(light.color),
                    );
                    let position = to_view_space(self.view, &light.position);
                    uniforms.insert(
                        program.location(&format!("{key}.position")).unwrap(),
                        Uniform::Vec3(position),
                    );
                    point_count += 1;
                }
            }
        }
        uniforms.insert(
            program.location("uPointLightsSize").unwrap(),
            Uniform::Int(point_count),
        );
        uniforms
    }

    fn add_model_uniforms(
        &self,
        program: &Program,
        uniforms: &mut Uniforms,
        model: &ModelNode,
        transform: &Transform,
        texture_count: &mut Count,
    ) {
        let program = &self.state.programs[model.program];
        uniforms.insert(
            program.location("uProjection").unwrap(),
            Uniform::Mat4(*self.projection),
        );
        uniforms.insert(
            program.location("uView").unwrap(),
            Uniform::Mat4(*self.view),
        );
        let model_matrix = transform.model_matrix();
        uniforms.insert(
            program.location("uModel").unwrap(),
            Uniform::Mat4(model_matrix),
        );
        let normal_matrix = Self::create_normal_matrix(&model_matrix, self.view);
        uniforms.insert(
            program.location("uNormal").unwrap(),
            Uniform::Mat4(normal_matrix),
        );
        let albedo = &self.state.textures[model.material.albedo];
        albedo.bind(texture_count.increment());
        uniforms.insert(
            program.location("uMaterial.albedo").unwrap(),
            Uniform::Int(texture_count.last() as i32),
        );
        let metallic_roughness_ao = &self.state.textures[model.material.metallic_roughness_ao];
        metallic_roughness_ao.bind(texture_count.increment());
        uniforms.insert(
            program.location("uMaterial.metallicRoughnessAo").unwrap(),
            Uniform::Int(texture_count.last() as i32),
        );
        uniforms.insert(program.location("uExposure").unwrap(), Uniform::Float(0.5));
    }

    fn add_skybox_uniforms(
        &self,
        program: &Program,
        uniforms: &mut Uniforms,
        model: &ModelNode,
        texture_count: &mut Count,
    ) {
        let program = &self.state.programs[model.program];
        let diffuse = &self.state.textures[self.skybox.diffuse];
        diffuse.bind(texture_count.increment());
        uniforms.insert(
            program.location("uIrradianceMap").unwrap(),
            Uniform::Int(texture_count.last() as i32),
        );
        let specular = &self.state.textures[self.skybox.specular];
        specular.bind(texture_count.increment());
        uniforms.insert(
            program.location("uPrefilterMap").unwrap(),
            Uniform::Int(texture_count.last() as i32),
        );
        let brdf_lut = &self.state.textures[self.skybox.brdf_lut];
        brdf_lut.bind(texture_count.increment());
        uniforms.insert(
            program.location("uBrdfLut").unwrap(),
            Uniform::Int(texture_count.last() as i32),
        );
    }
}
struct SkyboxShader<'a> {
    state: &'a State,
    view: &'a math::Matrix4,
    projection: &'a math::Matrix4,
    skybox: &'a Skybox,
}
impl<'a> OpenGlShader for SkyboxShader<'a> {
    fn render(&self) {
        unsafe {
            self.state.vertex_arrays[self.skybox.vao].bind();
            let skybox = &self.state.textures[self.skybox.image];
            skybox.bind(0);
            let program = &self.state.programs[self.skybox.program];
            program.use_and_set(&indexmap! {
                program.location("uSkybox").unwrap() => Uniform::Int(self.skybox.image as i32),
                program.location("uView").unwrap() => Uniform::Mat4(*self.view),
                program.location("uProjection").unwrap() => Uniform::Mat4(*self.projection),
                program.location("uGamma").unwrap() => Uniform::Float(2.2),
                program.location("uExposure").unwrap() => Uniform::Float(0.1),
            });
            gl::DepthFunc(gl::LEQUAL);
            gl::DrawArrays(gl::TRIANGLES, 0, Skybox::VERTICES.len() as i32);
            gl::DepthFunc(gl::LESS);
        }
    }
}
struct LineShader<'a> {
    state: &'a State,
    lines: &'a [Line],
    vao: &'a VertexArray,
    projection: &'a math::Matrix4,
    view: &'a math::Matrix4,
}
impl<'a> OpenGlShader for LineShader<'a> {
    fn render(&self) {
        unsafe {
            gl::Enable(gl::LINE_SMOOTH);
            self.vao.bind();
            for line in self.lines {
                let program = &self.state.programs[line.program];
                let model_matrix =
                    math::Matrix4::translation(&line.transform.position) * line.transform.rotation;
                program.use_and_set(&indexmap! {
                    program.location("uProjection").unwrap() => Uniform::Mat4(*self.projection),
                    program.location("uView").unwrap() => Uniform::Mat4(*self.view),
                    program.location("uModel").unwrap() => Uniform::Mat4(model_matrix),
                    program.location("uColor").unwrap() => Uniform::Vec3(vec3!(1.0, 0.0, 0.0))
                });
                gl::LineWidth(2.0);
                gl::DrawArrays(gl::LINES, 0, Self::LINE_VERTICES_LEN);
            }
            gl::Disable(gl::LINE_SMOOTH);
        }
    }
}

impl<'a> LineShader<'a> {
    pub const LINE_VERTICES_LEN: i32 = 2;
}

fn to_view_space(view: &math::Matrix4, vec: &math::Vec3) -> math::Vec3 {
    let res = view * vec.into_vec_with(1.0);
    res.into_vec()
}

#[derive(Debug, Clone)]
struct Skybox {
    vao: Idx,
    program: Idx,
    image: Idx,
    diffuse: Idx,
    specular: Idx,
    brdf_lut: Idx,
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
