use std::{ptr::null, sync::Arc};

use indexmap::IndexMap;

use crate::{
    gfx::{
        ModelNode, Shader,
        opengl::{
            OpenGlRenderer, ShaderSource, assert_no_ogl_error,
            shaders::{
                Draw, Framebuffer, OpenGlShader, OpenGlShaderImpl, Texture, TextureFormat,
                TextureParams, Uniform,
            },
        },
    },
    math, vec3,
    x3d::Transform,
};

pub struct GlobalShadowShader {
    shader: OpenGlShaderImpl,
}
impl OpenGlShader for GlobalShadowShader {
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
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            self.shader.render();
            gl::Viewport(params[0], params[1], params[2], params[3]);
            assert_no_ogl_error("render global shadows");
        }
    }
}
impl GlobalShadowShader {
    pub fn new(
        config: &GlobalShadowShaderConfig,
        model: &ModelNode,
        model_transform: &Transform,
    ) -> Self {
        let uniforms = indexmap::indexmap! {
            "uLightSpaceTransform".into() => Uniform::Matrix4(config.light_space_transform),
            "uModel".into() => Uniform::Matrix4(model_transform.model_matrix())
        };
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
        shader.set_render_target(config.render_target.clone());
        Self { shader }
    }
}

pub struct GlobalShadowShaderConfig {
    shader: Shader,
    render_target: Arc<Framebuffer>,
    shadow_map: Arc<Texture>,
    light_space_transform: math::Matrix4,
}
impl GlobalShadowShaderConfig {
    pub const SHADOW_VERTEX_SHADER: &str = include_str!("global_shadow.vs.glsl");
    pub const SHADOW_FRAGMENT_SHADER: &str = include_str!("global_shadow.fs.glsl");
    const NEAR_PLANE: f32 = 1.0;
    const FAR_PLANE: f32 = 7.5;

    pub fn init(light_direction: &math::Vec3, texture_unit: u32) -> Self {
        let (render_target, shadow_map) = Self::create_shadow_map_buffer(texture_unit);
        Self {
            shader: OpenGlRenderer::compile_shader(&[
                ShaderSource::new(Self::SHADOW_VERTEX_SHADER, gl::VERTEX_SHADER),
                ShaderSource::new(Self::SHADOW_FRAGMENT_SHADER, gl::FRAGMENT_SHADER),
            ])
            .expect("failed to compile global shadow shader"),
            render_target,
            shadow_map,
            light_space_transform: Self::get_light_space_transform(light_direction),
        }
    }

    fn create_shadow_map_buffer(texture_unit: u32) -> (Arc<Framebuffer>, Arc<Texture>) {
        let render_target = Arc::new(Framebuffer::new());
        let shadow_map = Texture::new(
            texture_unit,
            &TextureFormat {
                internalformat: gl::RGB as i32,
                format: gl::RGB,
                width: ShadowShaderConfig::SHADOW_MAP_SIZE,
                height: ShadowShaderConfig::SHADOW_MAP_SIZE,
            },
            &TextureParams {
                min_filter: gl::NEAREST as i32,
                mag_filter: gl::NEAREST as i32,
                wrap_t: gl::REPEAT as i32,
                wrap_s: gl::REPEAT as i32,
            },
            render_target.clone(),
        );
        (render_target, Arc::new(shadow_map))
    }

    fn get_light_space_transform(light_direction: &math::Vec3) -> math::Matrix4 {
        let projection =
            math::orthographic(-10.0, 10.0, -10.0, 10.0, Self::NEAR_PLANE, Self::FAR_PLANE);
        let view = math::look_at(
            &-light_direction,
            &vec3!(0.0, 0.0, 0.0),
            &vec3!(0.0, 1.0, 0.0),
        );
        projection * view
    }

    pub fn shadow_map(&self) -> Arc<Texture> {
        self.shadow_map.clone()
    }

    pub fn light_space_transform(&self) -> &math::Matrix4 {
        &self.light_space_transform
    }
}

pub struct ShadowShader {
    shader: OpenGlShaderImpl,
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
impl ShadowShader {
    pub const FRUSTUM_FAR: f32 = 25.0;
    pub const LIGHT_POS: math::Vec3 = vec3!(0.0, 1.0, 0.0);

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
        shader.set_render_target(config.buffer);
        Self { shader }
    }

    fn insert_shadow_transform_uniforms(
        uniforms: &mut IndexMap<String, Uniform>,
        light_position: &math::Vec3,
    ) {
        let projection = math::perspective(90.0, 1.0, 1.0, Self::FRUSTUM_FAR);
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ShadowShaderConfig {
    shader: Shader,
    buffer: Arc<Framebuffer>,
    texture: Arc<Texture>,
}
impl ShadowShaderConfig {
    pub const SHADOW_VERTEX_SHADER: &str = include_str!("shadow.vs.glsl");
    pub const SHADOW_GEOMETRY_SHADER: &str = include_str!("shadow.gs.glsl");
    pub const SHADOW_FRAGMENT_SHADER: &str = include_str!("shadow.fs.glsl");
    const SHADOW_MAP_SIZE: i32 = 1024;

    /// Prepares resources in OpenGL state machine to pass to [ShadowShader]
    pub fn init(texture_unit: u32) -> Self {
        let shader = OpenGlRenderer::compile_shader(&[
            ShaderSource::new(Self::SHADOW_VERTEX_SHADER, gl::VERTEX_SHADER),
            ShaderSource::new(Self::SHADOW_GEOMETRY_SHADER, gl::GEOMETRY_SHADER),
            ShaderSource::new(Self::SHADOW_FRAGMENT_SHADER, gl::FRAGMENT_SHADER),
        ])
        .expect("failed to compile shadow map shaders");
        let (buffer, texture) = Self::create_shadow_map_buffer(texture_unit);
        Self {
            shader,
            buffer,
            texture,
        }
    }

    fn create_shadow_map_buffer(texture_unit: u32) -> (Arc<Framebuffer>, Arc<Texture>) {
        let render_target = Arc::new(Framebuffer::new());
        let shadow_map = Texture::new(
            texture_unit,
            &TextureFormat {
                internalformat: gl::RGB as i32,
                format: gl::RGB,
                width: ShadowShaderConfig::SHADOW_MAP_SIZE,
                height: ShadowShaderConfig::SHADOW_MAP_SIZE,
            },
            &TextureParams {
                min_filter: gl::NEAREST as i32,
                mag_filter: gl::NEAREST as i32,
                wrap_t: gl::REPEAT as i32,
                wrap_s: gl::REPEAT as i32,
            },
            render_target.clone(),
        );
        (render_target, Arc::new(shadow_map))
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

    pub fn texture(&self) -> Arc<Texture> {
        self.texture.clone()
    }
}
