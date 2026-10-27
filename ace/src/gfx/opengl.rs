use indexmap::indexmap;
use std::ptr::null;

use crate::gfx::{opengl::state::*, *};

pub use state::Idx;

mod state;

#[allow(unused, reason = "used for debugging/pinpointing OpenGL error cause")]
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
        let vao = self.set_mesh_vao(node);
        let albedo = Texture::new(
            &node.albedo,
            gl::UNSIGNED_BYTE,
            &TextureFormat::SRGB_ALPHA,
            true,
        );
        let albedo = self.state.textures.insert(albedo);
        let metallic_roughness_ao = Texture::new(
            &node.metallic_roughness_ao,
            gl::UNSIGNED_BYTE,
            &TextureFormat::RGB,
            true,
        );
        let metallic_roughness_ao = self.state.textures.insert(metallic_roughness_ao);
        ModelNode {
            vao,
            indices: node.indices.as_ref().unwrap_or(&vec![]).len(),
            program,
            material: super::Material {
                albedo,
                metallic_roughness_ao,
            },
            vertices: node.vertices.len(),
        }
    }

    fn set_mesh_vao(&mut self, mesh: &MeshNode) -> usize {
        let vao = VertexArray::new(&mesh.vertices, mesh.indices.as_deref());
        self.state.vertex_arrays.insert(vao)
    }

    pub fn compile_shader(
        &mut self,
        vertex_shader: &str,
        fragment_shader: &str,
        tonemap_shader: &str,
    ) -> Result<Idx, OpenGlError> {
        let vertex_shader = Shader::compile(vertex_shader, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::compile(fragment_shader, gl::FRAGMENT_SHADER)?;
        let tonemap_shader = Shader::compile(tonemap_shader, gl::FRAGMENT_SHADER)?;
        let program = Program::link(&[vertex_shader, fragment_shader, tonemap_shader])?;
        let program = self.state.programs.insert(program);
        Ok(program)
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
                    self.state.vertex_arrays[node.vao].bind();
                    let program = &self.state.programs[node.program];
                    let uniforms = self.get_model_shader_uniforms(program, node, model);
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
impl<'a> ModelShader<'a> {
    fn create_normal_matrix(model: &math::Matrix4, view: &math::Matrix4) -> math::Matrix4 {
        (model * view).inverse().transpose()
    }

    fn get_model_shader_uniforms(
        &self,
        program: &Program,
        node: &ModelNode,
        model: &Model,
    ) -> IndexMap<Location, Uniform> {
        let mut uniforms = IndexMap::new();
        let mut texture_units = Count::init();
        self.add_model_uniforms(&mut uniforms, node, &model.transform, &mut texture_units);
        self.add_skybox_uniforms(&mut uniforms, node, &mut texture_units);
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
        uniforms: &mut Uniforms,
        model: &ModelNode,
        transform: &Transform,
        texture_units: &mut Count,
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
        albedo.bind(texture_units.increment());
        uniforms.insert(
            program.location("uMaterial.albedo").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        let metallic_roughness_ao = &self.state.textures[model.material.metallic_roughness_ao];
        metallic_roughness_ao.bind(texture_units.increment());
        uniforms.insert(
            program.location("uMaterial.metallicRoughnessAo").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        uniforms.insert(program.location("uExposure").unwrap(), Uniform::Float(0.5));
    }

    fn add_skybox_uniforms(
        &self,
        uniforms: &mut Uniforms,
        model: &ModelNode,
        texture_units: &mut Count,
    ) {
        let program = &self.state.programs[model.program];
        let diffuse = &self.state.textures[self.skybox.diffuse];
        diffuse.bind(texture_units.increment());
        uniforms.insert(
            program.location("uIrradianceMap").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        let specular = &self.state.textures[self.skybox.specular];
        specular.bind(texture_units.increment());
        uniforms.insert(
            program.location("uPrefilterMap").unwrap(),
            Uniform::Int(texture_units.last() as i32),
        );
        let brdf_lut = &self.state.textures[self.skybox.brdf_lut];
        brdf_lut.bind(texture_units.increment());
        uniforms.insert(
            program.location("uBrdfLut").unwrap(),
            Uniform::Int(texture_units.last() as i32),
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
