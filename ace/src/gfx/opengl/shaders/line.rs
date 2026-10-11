use crate::{
    gfx::{
        Line, VertexArray,
        opengl::shaders::{Draw, OpenGlShader, OpenGlShaderImpl, Uniform},
    },
    math, vec3,
};

pub struct LineShader {
    shader: OpenGlShaderImpl,
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
