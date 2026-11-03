use indexmap::indexmap;
use std::ptr::null;
use std::sync::Arc;

use super::state::*;
use crate::gfx::*;

pub static TONEMAPPING_SHADER: &str = include_str!("tonemapping.glsl");

pub trait OpenGlShader {
    fn render(&self, state: &State, renderables: &[Renderable], view_projection: &ViewProjection);
}

pub struct ViewProjection {
    pub view: math::Matrix4,
    pub projection: math::Matrix4,
}

pub struct ModelShader {
    skybox: Arc<Skybox>,
    program: Program,
}
impl ModelShader {
    const VERTEX_SHADER_PBR: &str = include_str!("pbr.vs.glsl");
    const FRAGMENT_SHADER_PBR: &str = include_str!("pbr.fs.glsl");

    pub fn new(skybox: Arc<Skybox>) -> Self {
        let program = Program::link(&[
            Shader::compile(Self::VERTEX_SHADER_PBR, gl::VERTEX_SHADER)
                .expect("failed to compile pbr vertex shader"),
            Shader::compile(Self::FRAGMENT_SHADER_PBR, gl::FRAGMENT_SHADER)
                .expect("failed to compile pbr fragment shader"),
            Shader::compile(TONEMAPPING_SHADER, gl::FRAGMENT_SHADER)
                .expect("failed to compile tonemapping shader"),
        ])
        .expect("failed to link pbr shader program");
        Self { skybox, program }
    }
}
impl OpenGlShader for ModelShader {
    fn render(&self, state: &State, renderables: &[Renderable], view_projection: &ViewProjection) {
        unsafe {
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
            for model in &models {
                for node in &model.nodes {
                    state.vertex_arrays[node.vao].bind();
                    let program = &self.program;
                    let uniforms = self.get_model_shader_uniforms(
                        state,
                        &lights,
                        program,
                        node,
                        model,
                        view_projection,
                    );
                    program.use_and_set(&uniforms);
                    if node.indices > 0 {
                        gl::DrawElements(
                            gl::TRIANGLES,
                            node.indices as i32,
                            gl::UNSIGNED_INT,
                            null(),
                        );
                    } else {
                        gl::DrawArrays(gl::TRIANGLES, 0, node.vertices as i32);
                    }
                }
            }
        }
    }
}
impl ModelShader {
    fn get_model_shader_uniforms(
        &self,
        state: &State,
        lights: &[Light],
        program: &Program,
        node: &ModelNode,
        model: &Model,
        view_projection: &ViewProjection,
    ) -> IndexMap<Location, Uniform> {
        let mut uniforms = IndexMap::new();
        let mut texture_units = Count::init();
        self.add_model_uniforms(
            state,
            &mut uniforms,
            node,
            &model.transform,
            &mut texture_units,
            view_projection,
        );
        self.add_skybox_uniforms(&mut uniforms, &mut texture_units);
        let mut point_count = 0;
        for light in lights {
            match light {
                Light::Point(light) => {
                    let key = &format!("uPointLights[{point_count}]");
                    uniforms.insert(
                        program.location(&format!("{key}.color")).unwrap(),
                        Uniform::Vec3(light.color),
                    );
                    let position = Self::to_view_space(&view_projection.view, &light.position);
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

    fn to_view_space(view: &math::Matrix4, vec: &math::Vec3) -> math::Vec3 {
        let res = view * vec.into_vec_with(1.0);
        res.into_vec()
    }

    fn add_model_uniforms(
        &self,
        state: &State,
        uniforms: &mut Uniforms,
        model: &ModelNode,
        transform: &Transform,
        texture_units: &mut Count,
        view_projection: &ViewProjection,
    ) {
        uniforms.insert(
            self.program.location("uProjection").unwrap(),
            Uniform::Mat4(view_projection.projection),
        );
        uniforms.insert(
            self.program.location("uView").unwrap(),
            Uniform::Mat4(view_projection.view),
        );
        let model_matrix = transform.model_matrix();
        uniforms.insert(
            self.program.location("uModel").unwrap(),
            Uniform::Mat4(model_matrix),
        );
        let normal_matrix = Self::create_normal_matrix(&model_matrix, &view_projection.view);
        uniforms.insert(
            self.program.location("uNormal").unwrap(),
            Uniform::Mat4(normal_matrix),
        );
        let albedo = &state.textures[model.material.albedo];
        albedo.bind(texture_units.increment());
        uniforms.insert(
            self.program.location("uMaterial.albedo").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        let metallic_roughness_ao = &state.textures[model.material.metallic_roughness_ao];
        metallic_roughness_ao.bind(texture_units.increment());
        uniforms.insert(
            self.program
                .location("uMaterial.metallicRoughnessAo")
                .unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        uniforms.insert(
            self.program.location("uExposure").unwrap(),
            Uniform::Float(0.5),
        );
    }

    fn create_normal_matrix(model: &math::Matrix4, view: &math::Matrix4) -> math::Matrix4 {
        (model * view).inverse().transpose()
    }

    fn add_skybox_uniforms(&self, uniforms: &mut Uniforms, texture_units: &mut Count) {
        self.skybox.diffuse.bind(texture_units.increment());
        uniforms.insert(
            self.program.location("uIrradianceMap").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        self.skybox.specular.bind(texture_units.increment());
        uniforms.insert(
            self.program.location("uPrefilterMap").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        self.skybox.brdf_lut.bind(texture_units.increment());
        uniforms.insert(
            self.program.location("uBrdfLut").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
    }
}

pub struct SkyboxShader {
    skybox: Arc<Skybox>,
    program: Program,
}
impl SkyboxShader {
    const VERTEX_SHADER_SKYBOX: &str = include_str!("skybox.vs.glsl");
    const FRAGMENT_SHADER_SKYBOX: &str = include_str!("skybox.fs.glsl");

    pub fn new(skybox: Arc<Skybox>) -> Self {
        let program = Program::link(&[
            Shader::compile(Self::VERTEX_SHADER_SKYBOX, gl::VERTEX_SHADER)
                .expect("failed to compile skybox vertex shader"),
            Shader::compile(Self::FRAGMENT_SHADER_SKYBOX, gl::FRAGMENT_SHADER)
                .expect("failed to compile skybox fragment shader"),
            Shader::compile(TONEMAPPING_SHADER, gl::FRAGMENT_SHADER)
                .expect("failed to compile tonemapping shader"),
        ])
        .expect("failed to link skybox shader program");
        Self { skybox, program }
    }
}
impl OpenGlShader for SkyboxShader {
    fn render(&self, _: &State, _: &[Renderable], view_projection: &ViewProjection) {
        unsafe {
            self.skybox.vao.bind();
            self.skybox.image.bind(0);
            let view = Self::remove_translation(&view_projection.view);
            self.program.use_and_set(&indexmap! {
                self.program.location("uSkybox").unwrap() => Uniform::Int(0),
                self.program.location("uView").unwrap() => Uniform::Mat4(view),
                self.program.location("uProjection").unwrap() => Uniform::Mat4(view_projection.projection),
                self.program.location("uGamma").unwrap() => Uniform::Float(2.2),
                self.program.location("uExposure").unwrap() => Uniform::Float(0.1),
            });
            gl::DepthFunc(gl::LEQUAL);
            gl::DrawArrays(gl::TRIANGLES, 0, Skybox::VERTICES.len() as i32);
            gl::DepthFunc(gl::LESS);
        }
    }
}
impl SkyboxShader {
    fn remove_translation(view: &math::Matrix4) -> math::Matrix4 {
        [
            [view[0][0], view[0][1], view[0][2], 0.0],
            [view[1][0], view[1][1], view[1][2], 0.0],
            [view[2][0], view[2][1], view[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into()
    }
}
#[derive(Debug)]
pub struct Skybox {
    vao: VertexArray,
    image: Texture,
    diffuse: Texture,
    specular: Texture,
    brdf_lut: Texture,
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

    pub fn new(skybox: &gfx::Ibl) -> Self {
        let vao = VertexArray::simple_new(&Skybox::VERTICES);
        let image = Texture::new(&skybox.skybox, gl::FLOAT, &TextureFormat::RGB32F, true);
        image.texture_filter(gl::LINEAR as i32, gl::LINEAR as i32);
        let diffuse = Texture::new(&skybox.diffuse, gl::FLOAT, &TextureFormat::RGB32F, true);
        let specular =
            Texture::new_with_levels(&skybox.specular, gl::FLOAT, &TextureFormat::RGB32F);
        specular.texture_filter(gl::LINEAR_MIPMAP_LINEAR as i32, gl::LINEAR as i32);
        let brdf_lut = Texture::new(
            &skybox.brdf_lut,
            gl::UNSIGNED_BYTE,
            &TextureFormat::RGB,
            true,
        );
        Self {
            vao,
            image,
            specular,
            diffuse,
            brdf_lut,
        }
    }
}

pub struct LineShader {
    line: VertexArray,
    program: Program,
}
impl Default for LineShader {
    fn default() -> Self {
        Self::new()
    }
}
impl LineShader {
    const LINE: [math::Vec3; 2] = [vec3!(0.0), vec3!(0.0, 0.0, -100.0)];
    const LINE_VERTICES_LEN: i32 = 2;

    const VERTEX_SHADER_LINE: &str = include_str!("line.vs.glsl");
    const FRAGMENT_SHADER_LINE: &str = include_str!("line.fs.glsl");

    pub fn new() -> Self {
        let line = VertexArray::simple_new(&Self::LINE);
        let program = Program::link(&[
            Shader::compile(Self::VERTEX_SHADER_LINE, gl::VERTEX_SHADER)
                .expect("failed to compile line vertex shader"),
            Shader::compile(Self::FRAGMENT_SHADER_LINE, gl::FRAGMENT_SHADER)
                .expect("failed to compile line fragment shader"),
        ])
        .expect("failed to link line shader program");
        Self { line, program }
    }
}
impl OpenGlShader for LineShader {
    fn render(&self, _: &State, renderables: &[Renderable], view_projection: &ViewProjection) {
        let lines: Vec<Line> = renderables
            .iter()
            .filter_map(|r| maybe_component!(r, Renderable::Line))
            .cloned()
            .collect();
        unsafe {
            gl::Enable(gl::LINE_SMOOTH);
            self.line.bind();
            for line in &lines {
                let model_matrix =
                    math::Matrix4::translation(&line.transform.position) * line.transform.rotation;
                self.program.use_and_set(&indexmap! {
                self.program.location("uProjection").unwrap() => Uniform::Mat4(view_projection.projection),
                self.program.location("uView").unwrap() => Uniform::Mat4(view_projection.view),
                self.program.location("uModel").unwrap() => Uniform::Mat4(model_matrix),
                self.program.location("uColor").unwrap() => Uniform::Vec3(vec3!(1.0, 0.0, 0.0))
            });
                gl::LineWidth(2.0);
                gl::DrawArrays(gl::LINES, 0, Self::LINE_VERTICES_LEN);
                // vertex array
            }
            gl::Disable(gl::LINE_SMOOTH);
        }
    }
}
