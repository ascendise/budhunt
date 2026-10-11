use crate::{
    gfx::{
        Light, ModelNode, Tex,
        opengl::{
            Skybox,
            shaders::{
                Draw, OpenGlShader, OpenGlShaderImpl, Uniform,
                shadow::{GlobalShadowShaderConfig, ShadowShader},
            },
        },
    },
    math,
    x3d::Transform,
};

pub struct ModelShader {
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
        global_shadow_config: &GlobalShadowShaderConfig,
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
            "uIrradianceMap".into() => Uniform::Int(skybox.diffuse()),
            "uPrefilterMap".into() => Uniform::Int(skybox.specular()),
            "uBrdfLut".into() => Uniform::Int(skybox.brdf_lut()),
            "uLightPos".into() => Uniform::Vec3(light_pos.into_vec()), //TODO: pass light position
            "uLightSpaceTransform".into() => Uniform::Matrix4(*global_shadow_config.light_space_transform()),
            "uGlobalShadowMap".into() => Uniform::Int(global_shadow_config.shadow_map())
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
