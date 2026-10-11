use crate::{
    gfx::{
        Shader, Tex, VertexArray,
        opengl::shaders::{Draw, OpenGlShader, OpenGlShaderImpl, Uniform},
    },
    math, vec3,
};

pub struct SkyboxShader {
    shader: OpenGlShaderImpl,
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

    pub fn new(
        vao: VertexArray,
        shader: Shader,
        image: Tex,
        diffuse: Tex,
        specular: Tex,
        brdf_lut: Tex,
    ) -> Self {
        Self {
            vao,
            shader,
            image,
            diffuse,
            specular,
            brdf_lut,
        }
    }

    pub fn shader(&self, projection: math::Matrix4, view: math::Matrix4) -> SkyboxShader {
        let view: math::Matrix4 = [
            [view[0][0], view[0][1], view[0][2], 0.0],
            [view[1][0], view[1][1], view[1][2], 0.0],
            [view[2][0], view[2][1], view[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into();
        SkyboxShader::new(self, projection, view)
    }

    pub fn diffuse(&self) -> i32 {
        self.diffuse
    }

    pub fn specular(&self) -> i32 {
        self.specular
    }

    pub fn brdf_lut(&self) -> i32 {
        self.brdf_lut
    }
}
