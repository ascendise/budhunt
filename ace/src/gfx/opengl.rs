use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ptr::{null, null_mut},
};

use crate::gfx::*;

pub struct OpenGlRenderer {
    texture_count: u32,
    skybox: Option<Skybox>,
    line_vao: VertexArray,
    shadow_config: ShadowShaderConfig,
}
impl Renderer for OpenGlRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, renderables: &[Renderable]) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let projection = &projection.to_projection_matrix();
        let view = &camera.to_view_matrix();
        let skybox = self.skybox.as_ref().expect("No skybox set!");
        let lights: Vec<&Light> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Light))
            .collect();
        let models: Vec<Model> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Model))
            .cloned()
            .collect();
        for model in &models {
            for node in &model.nodes {
                let shadow_shader =
                    ShadowShader::new(self.shadow_config.clone(), node, &model.transform);
                shadow_shader.render();
            }
        }
        for model in models {
            for node in model.nodes {
                let shader = ModelShader::new(
                    &node,
                    &model.transform,
                    *projection,
                    *view,
                    &lights,
                    skybox,
                    self.shadow_config.texture,
                );
                shader.render();
            }
        }
        skybox.shader(*projection, *view).render();
        self.render_lines(*projection, *view, renderables);
    }
}
impl OpenGlRenderer {
    pub const LINE: [math::Vec3; 2] = [vec3!(0.0), vec3!(0.0, 0.0, -100.0)];

    pub fn init() -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            let line_vao = Self::init_line();
            Self {
                texture_count: 0,
                skybox: None,
                line_vao,
                shadow_config: ShadowShaderConfig::init(),
            }
        }
    }

    fn init_line() -> VertexArray {
        unsafe {
            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = size_of_val(&Self::LINE) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                Self::LINE.as_ptr() as *const _,
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
            vertex_array
        }
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh, shader: Shader) -> ModelNodes {
        mesh.nodes
            .iter()
            .map(|n| self.load_mesh_node(n, shader))
            .collect()
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

    pub fn compile_shader(shaders: &[ShaderSource]) -> Result<Shader, OpenGlError> {
        unsafe {
            let mut compiled_shaders = vec![];
            let shader_program = gl::CreateProgram();
            for shader in shaders {
                let shader = Self::compile(shader.source, shader.shader_type);
                match shader {
                    Ok(s) => {
                        compiled_shaders.push(s);
                        gl::AttachShader(shader_program, s);
                    }
                    Err(e) => {
                        Self::delete_shaders(&compiled_shaders);
                        return Err(e);
                    }
                }
            }
            gl::LinkProgram(shader_program);
            let mut success = 0;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetProgramInfoLog(shader_program, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                Self::delete_shaders(&compiled_shaders);
                return Err(OpenGlError { err });
            }
            Self::delete_shaders(&compiled_shaders);
            Ok(shader_program)
        }
    }

    fn delete_shaders(shaders: &[u32]) {
        unsafe {
            shaders.iter().for_each(|s| gl::DeleteShader(*s));
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

    fn render_lines(
        &self,
        projection: math::Matrix4,
        view: math::Matrix4,
        renderables: &[Renderable],
    ) {
        let lines: Vec<Line> = renderables
            .iter()
            .filter_map(|r| maybe_component!(r, Renderable::Line))
            .cloned()
            .collect();
        for line in lines {
            let shader = LineShader::new(projection, view, line, self.line_vao);
            shader.render();
        }
    }
}

pub struct ShaderSource {
    source: &'static str,
    shader_type: u32,
}
impl ShaderSource {
    pub fn new(source: &'static str, shader_type: u32) -> Self {
        Self {
            source,
            shader_type,
        }
    }
}

trait OpenGlShader {
    fn render(&self);
}
struct OpenGlShaderImpl {
    vao: VertexArray,
    shader: Shader,
    uniforms: IndexMap<String, Uniform>,
    draw: Draw,
    framebuffer: Framebuffer,
}
impl OpenGlShader for OpenGlShaderImpl {
    fn render(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.shader);
            self.set_uniforms();
            self.draw.draw();
        }
    }
}
impl OpenGlShaderImpl {
    fn new(
        vao: VertexArray,
        shader: Shader,
        uniforms: IndexMap<String, Uniform>,
        draw: Draw,
    ) -> Self {
        Self {
            vao,
            shader,
            draw,
            uniforms,
            framebuffer: 0,
        }
    }

    fn set_uniforms(&self) {
        for (key, uniform) in &self.uniforms {
            uniform.set(self.shader, key);
        }
    }

    fn set_framebuffer(&mut self, framebuffer: Framebuffer) {
        self.framebuffer = framebuffer;
    }
}
enum Uniform {
    Float(f32),
    Int(i32),
    Vec3(math::Vec3),
    Matrix4(math::Matrix4),
}
impl Uniform {
    pub fn set(&self, shader: Shader, key: &str) {
        match self {
            Uniform::Float(f) => Self::set_float_uniform(shader, *f, key),
            Uniform::Int(i) => Self::set_int_uniform(shader, *i, key),
            Uniform::Vec3(v3) => Self::set_vec3_uniform(shader, v3, key),
            Uniform::Matrix4(m4) => Self::set_matrix_uniform(shader, m4, key),
        }
    }

    fn set_matrix_uniform(shader: Shader, matrix: &math::Matrix4, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::UniformMatrix4fv(location, 1, gl::TRUE, matrix.data.as_ptr() as *const _) }
    }

    fn set_vec3_uniform(shader: Shader, value: &math::Vec3, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform3f(location, value.x(), value.y(), value.z()) }
    }

    fn set_int_uniform(shader: Shader, value: i32, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform1i(location, value) }
    }

    fn set_float_uniform(shader: Shader, value: f32, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform1f(location, value) }
    }

    fn uniform_location(shader: Shader, key: &str) -> gl::types::GLint {
        let key = CString::new(key).unwrap();
        unsafe { gl::GetUniformLocation(shader, key.as_ptr()) }
    }
}
enum Draw {
    Indices(i32),
    Vertices(i32),
    Line,
}
impl Draw {
    pub fn draw(&self) {
        unsafe {
            match self {
                Draw::Indices(i) => gl::DrawElements(gl::TRIANGLES, *i, gl::UNSIGNED_INT, null()),
                Draw::Vertices(i) => gl::DrawArrays(gl::TRIANGLES, 0, *i),
                Draw::Line => gl::DrawArrays(gl::LINES, 0, OpenGlRenderer::LINE.len() as i32),
            };
        }
    }
}
type Framebuffer = u32;

struct ModelShader {
    shader: OpenGlShaderImpl,
}
impl OpenGlShader for ModelShader {
    fn render(&self) {
        self.shader.render()
    }
}
impl ModelShader {
    pub fn new(
        model: &ModelNode,
        transform: &Transform,
        projection: math::Matrix4,
        view: math::Matrix4,
        lights: &[&Light],
        skybox: &Skybox,
        shadow_map: Tex,
    ) -> Self {
        let model_matrix = transform.model_matrix();
        let light_pos = view * model_matrix * ShadowShader::LIGHT_POS.into_vec();
        let mut uniforms = indexmap::indexmap! {
            "uView".into() => Uniform::Matrix4(view),
            "uProjection".into() => Uniform::Matrix4(projection),
            "uModel".into() => Uniform::Matrix4(model_matrix),
            "uNormal".into() => Uniform::Matrix4((model_matrix * view).inverse().transpose()),
            "uMaterial.albedo".into() => Uniform::Int(model.material.albedo),
            "uMaterial.metallicRoughnessAo".into() => Uniform::Int(model.material.metallic_roughness_ao),
            "uShadowMap".into() => Uniform::Int(shadow_map),
            "uExposure".into() => Uniform::Float(0.5),
            "uIrradianceMap".into() => Uniform::Int(skybox.diffuse),
            "uPrefilterMap".into() => Uniform::Int(skybox.specular),
            "uBrdfLut".into() => Uniform::Int(skybox.brdf_lut),
            "uLightPos".into() => Uniform::Vec3(light_pos.into_vec()), //TODO: pass light position
        };
        for (l, light) in lights.iter().enumerate() {
            let key = format!("uPointLights[{l}]");
            let Light::Point(light) = light;
            uniforms.insert(format!("{key}.color"), Uniform::Vec3(light.color));
            uniforms.insert(format!("{key}.position"), Uniform::Vec3(light.position));
        }
        uniforms.insert("uPointLightsSize".into(), Uniform::Int(lights.len() as i32));
        let draw = if model.indices > 0 {
            Draw::Indices(model.indices)
        } else {
            Draw::Vertices(model.vertices)
        };
        let shader = OpenGlShaderImpl::new(model.vao, model.shader, uniforms, draw);
        Self { shader }
    }
}

struct LineShader {
    shader: OpenGlShaderImpl,
}
impl LineShader {
    const COLOR_RED: math::Vec3 = vec3!(1.0, 0.0, 0.0);

    pub fn new(
        projection: math::Matrix4,
        view: math::Matrix4,
        line: Line,
        vao: VertexArray,
    ) -> Self {
        let shader = OpenGlShaderImpl::new(
            vao,
            line.shader,
            indexmap::indexmap! {
                "uProjection".into() => Uniform::Matrix4(projection),
                "uView".into() => Uniform::Matrix4(view),
                "uModel".into() => Uniform::Matrix4(line.transform.model_matrix()),
                "uColor".into() => Uniform::Vec3(Self::COLOR_RED)
            },
            Draw::Line,
        );
        Self { shader }
    }
}
impl OpenGlShader for LineShader {
    fn render(&self) {
        unsafe {
            gl::Enable(gl::LINE_SMOOTH);
            gl::LineWidth(2.0);
            self.shader.render();
            gl::Disable(gl::LINE_SMOOTH);
        }
    }
}
struct ShadowShader {
    shader: OpenGlShaderImpl,
}
impl ShadowShader {
    const FRUSTUM_FAR: f32 = 25.0;
    const LIGHT_POS: math::Vec<3> = vec3!(0.0, 1.0, 0.0);

    pub fn new(config: ShadowShaderConfig, model: &ModelNode, transform: &Transform) -> Self {
        let mut uniforms = indexmap::indexmap! {
            "uModel".into() => Uniform::Matrix4(transform.model_matrix()),
            "uFrustumFar".into() => Uniform::Float(Self::FRUSTUM_FAR),
            "uLightPos".into() => Uniform::Vec3(Self::LIGHT_POS), //TODO: pass light position
        };
        Self::insert_shadow_transform_uniforms(&mut uniforms, &Self::LIGHT_POS);
        let mut shader = OpenGlShaderImpl::new(
            model.vao,
            config.shader,
            uniforms,
            if model.indices > 0 {
                Draw::Indices(model.indices)
            } else {
                Draw::Vertices(model.vertices)
            },
        );
        shader.set_framebuffer(config.buffer);
        Self { shader }
    }

    fn insert_shadow_transform_uniforms(
        uniforms: &mut IndexMap<String, Uniform>,
        light_position: &math::Vec3,
    ) {
        let projection = math::projection(90.0, 1.0, 1.0, Self::FRUSTUM_FAR);
        let matrices = [
            math::look_at(
                light_position,
                &(light_position + vec3!(1.0, 0.0, 0.0)),
                &vec3!(0.0, -1.0, 0.0),
            ),
            math::look_at(
                light_position,
                &(light_position + vec3!(-1.0, 0.0, 0.0)),
                &vec3!(0.0, -1.0, 0.0),
            ),
            math::look_at(
                light_position,
                &(light_position + vec3!(0.0, 1.0, 0.0)),
                &vec3!(0.0, 0.0, 1.0),
            ),
            math::look_at(
                light_position,
                &(light_position + vec3!(0.0, -1.0, 0.0)),
                &vec3!(0.0, 0.0, -1.0),
            ),
            math::look_at(
                light_position,
                &(light_position + vec3!(0.0, 0.0, 1.0)),
                &vec3!(0.0, -1.0, 0.0),
            ),
            math::look_at(
                light_position,
                &(light_position + vec3!(0.0, 0.0, -1.0)),
                &vec3!(0.0, -1.0, 0.0),
            ),
        ];
        for (m, matrix) in matrices.iter().enumerate() {
            uniforms.insert(
                format!("uShadowTransforms[{m}]"),
                Uniform::Matrix4(&projection * matrix),
            );
        }
    }
}
impl OpenGlShader for ShadowShader {
    fn render(&self) {
        unsafe {
            let mut params = [0; 4];
            gl::GetIntegerv(gl::VIEWPORT, params.as_mut_ptr());
            gl::Viewport(
                0,
                0,
                ShadowShaderConfig::SHADOW_MAP_SIZE,
                ShadowShaderConfig::SHADOW_MAP_SIZE,
            );
            self.shader.render();
            gl::Viewport(params[0], params[1], params[2], params[3]);
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ShadowShaderConfig {
    shader: Shader,
    buffer: Framebuffer,
    texture: Tex,
}
impl ShadowShaderConfig {
    pub const SHADOW_VERTEX_SHADER: &str = include_str!("shadow.vs.glsl");
    pub const SHADOW_GEOMETRY_SHADER: &str = include_str!("shadow.gs.glsl");
    pub const SHADOW_FRAGMENT_SHADER: &str = include_str!("shadow.fs.glsl");
    const SHADOW_MAP_SIZE: i32 = 1024;

    /// Prepares resources in OpenGL state machine to pass to [ShadowShaderImpl]
    pub fn init() -> Self {
        let shader = OpenGlRenderer::compile_shader(&[
            ShaderSource::new(Self::SHADOW_VERTEX_SHADER, gl::VERTEX_SHADER),
            ShaderSource::new(Self::SHADOW_GEOMETRY_SHADER, gl::GEOMETRY_SHADER),
            ShaderSource::new(Self::SHADOW_FRAGMENT_SHADER, gl::FRAGMENT_SHADER),
        ])
        .expect("failed to compile shadow map shaders");
        let (buffer, texture) = Self::create_shadow_map();
        Self {
            shader,
            buffer,
            texture,
        }
    }

    fn create_shadow_map() -> (u32, Tex) {
        unsafe {
            let mut buffer = 0;
            gl::GenFramebuffers(1, &mut buffer);
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
            for face in 0..=5 {
                let face = gl::TEXTURE_CUBE_MAP_POSITIVE_X + face;
                Self::set_cubemap_tex(face);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, buffer);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, texture, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            let framebuffer_err = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            assert!(
                framebuffer_err == gl::FRAMEBUFFER_COMPLETE,
                "Incomplete framebuffer: {framebuffer_err:x}"
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            (buffer, texture as Tex)
        }
    }

    fn set_cubemap_tex(face: u32) {
        unsafe {
            gl::TexImage2D(
                face,
                0,
                gl::DEPTH_COMPONENT as i32,
                Self::SHADOW_MAP_SIZE,
                Self::SHADOW_MAP_SIZE,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                null(),
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );
        }
    }
}

#[derive(Debug, Clone)]
struct Skybox {
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

    fn shader(&self, projection: math::Matrix4, view: math::Matrix4) -> SkyboxShader {
        let view: math::Matrix4 = [
            [view[0][0], view[0][1], view[0][2], 0.0],
            [view[1][0], view[1][1], view[1][2], 0.0],
            [view[2][0], view[2][1], view[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into();
        SkyboxShader::new(self, projection, view)
    }
}
struct SkyboxShader {
    shader: OpenGlShaderImpl,
}
impl SkyboxShader {
    pub fn new(skybox: &Skybox, projection: math::Matrix4, view: math::Matrix4) -> Self {
        let uniforms = indexmap::indexmap! {
            "uSkybox".into() => Uniform::Int(skybox.image),
            "uView".into() => Uniform::Matrix4(view),
            "uProjection".into() => Uniform::Matrix4(projection),
            "uGamma".into() => Uniform::Float(2.2),
            "uExposure".into() => Uniform::Float(0.1),
        };
        let shader = OpenGlShaderImpl::new(
            skybox.vao,
            skybox.shader,
            uniforms,
            Draw::Vertices(Skybox::VERTICES.len() as i32),
        );
        Self { shader }
    }
}
impl OpenGlShader for SkyboxShader {
    fn render(&self) {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            self.shader.render();
            gl::DepthFunc(gl::LESS);
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

pub fn assert_no_ogl_error(message: &str) {
    unsafe {
        let mut is_err = false;
        loop {
            let err = gl::GetError();
            if err == 0 {
                break;
            }
            is_err = true;
            eprintln!("OPENGL ERROR ({err}): {message}");
        }
        assert!(!is_err, "OpenGL error encountered. Terminating program");
    }
}
