use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ops::Index,
    ptr::{null, null_mut},
};

use indexmap::IndexMap;

use crate::{
    gfx::{Image, Vertex},
    math,
};

pub type Idx = usize;
#[derive(Debug, PartialEq, Eq)]
pub struct State {
    pub vertex_arrays: StateCollection<VertexArray>,
    pub textures: StateCollection<Texture>,
    pub programs: StateCollection<Program>,
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
pub struct StateCollection<T> {
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
pub struct Count(isize);
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
pub struct Texture(u32);
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
pub struct TextureFormat(gl::types::GLint, gl::types::GLenum);
impl TextureFormat {
    pub const SRGB_ALPHA: TextureFormat = TextureFormat(gl::SRGB_ALPHA as i32, gl::RGBA);
    pub const RGB: TextureFormat = TextureFormat(gl::RGB8 as i32, gl::RGB);
    pub const RGB32F: TextureFormat = TextureFormat(gl::RGB32F as i32, gl::RGB);
}
#[derive(Debug, PartialEq, Eq)]
pub struct Program(u32);
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
pub type Uniforms = IndexMap<Location, Uniform>;
pub enum Uniform {
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

#[derive(Eq, Hash, PartialEq)]
pub struct Location(gl::types::GLint);
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

pub struct Shader(gl::types::GLuint);
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
