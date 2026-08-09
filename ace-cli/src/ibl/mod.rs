use std::{f32::consts::PI, sync::Arc, thread};

use crate::units::{Spheric, UV};
use ace::{
    gfx::{Ibl, Image},
    math, vec2, vec3,
};
use image::GenericImageView;

pub trait Baker {
    fn bake(&self, image: Arc<image::DynamicImage>) -> Ibl;
}

pub struct CpuBaker {
    cores: usize,
    convolute_sample_delta: f32,
    brdf_sample_count: u32,
}
impl Baker for CpuBaker {
    fn bake(&self, image: Arc<image::DynamicImage>) -> Ibl {
        let max_cores = match thread::available_parallelism() {
            Ok(v) => v.get(),
            Err(_) => {
                tracing::warn!("Could not get parallelism info. Defaulting to using 1 core!");
                1
            }
        };
        let mut cores = self.cores.max(max_cores) as i32;
        tracing::info!("Baking IBL with {cores} cores");
        thread::scope(|s| {
            let lut = s.spawn(|| tracing::info_span!("BRDF LUT").in_scope(|| self.bake_lut(512)));
            cores -= 1;
            let specular = s.spawn(|| {
                tracing::info_span!("Specular").in_scope(|| self.bake_specular(image.clone()))
            });
            cores -= 1;
            let diffuse = s.spawn(|| {
                tracing::info_span!("Diffuse")
                    .in_scope(|| self.bake_diffuse(image.clone(), cores.max(1) as usize))
            });
            Ibl {
                skybox: Image {
                    width: image.width(),
                    height: image.height(),
                    data: image.to_rgb32f().pixels().flat_map(|f| f.0).collect(),
                },
                diffuse: diffuse.join().expect("failure during diffuse baking"),
                specular: specular.join().expect("failure during specular baking"),
                brdf_lut: lut.join().expect("failure during lut baking"),
            }
        })
    }
}

impl CpuBaker {
    pub fn new(cores: usize, convolute_sample_delta: f32, brdf_sample_count: u32) -> Self {
        Self {
            cores,
            convolute_sample_delta,
            brdf_sample_count,
        }
    }

    fn bake_lut(&self, resolution: u32) -> Image {
        tracing::info!("Baking Lookup Texture");
        let mut lut = Vec::new();
        for y in 0..resolution {
            for x in 0..resolution {
                let uv = UV::from_screen_coordinates(x, y, resolution, resolution);
                let pixel = Self::integrate_brdf(uv.u, 1.0 - uv.v, self.brdf_sample_count);
                let pixel = pixel * 255.0;
                lut.push(pixel)
            }
        }
        let image = Self::to_image(&lut, resolution, resolution);
        tracing::info!("Finished generating Lookup Texture");
        image
    }

    fn integrate_brdf(direction: f32, roughness: f32, sample_count: u32) -> math::Vec3 {
        let v = vec3!((1.0 - direction.powi(2)).sqrt(), 0.0, direction);
        let front = vec3!(0.0, 0.0, 1.0);
        let mut pixel = vec3!(0.0);
        for i in 0..sample_count {
            let xi = Self::hammersley(i, sample_count);
            let h = Self::importance_sampling_ggx(xi, &front, roughness);
            let l = &(&h * (2.0 * v.dot(&h))) - &v;
            let l = l.normalize();
            let nl = l.z.max(0.0);
            let nh = h.z.max(0.0);
            let vh = v.dot(&h).max(0.0);
            if nl > 0.0 {
                let g = Self::geometry_smith(&front, &v, &l, roughness);
                let g_vis = (g * vh) / (nh * direction);
                let fc = (1.0 - vh).powi(5);
                pixel.x += (1.0 - fc) * g_vis;
                pixel.y += fc * g_vis;
            }
        }
        pixel.x /= sample_count as f32;
        pixel.y /= sample_count as f32;
        pixel
    }

    fn hammersley(i: u32, sample_count: u32) -> math::Vec2 {
        vec2!(
            i as f32 / sample_count as f32,
            Self::radical_inverse_van_der_corput(i)
        )
    }

    fn importance_sampling_ggx(xi: math::Vec2, normal: &math::Vec3, roughness: f32) -> math::Vec3 {
        let roughness = roughness.powi(2);
        let phi = 2.0 * PI * xi.x;
        let cos_theta = f32::sqrt((1.0 - xi.y) / (1.0 + (roughness.powi(2) - 1.0) * xi.y));
        let sin_theta = f32::sqrt(1.0 - cos_theta.powi(2));
        let h = vec3!(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta);
        let up = if normal.z.abs() < 0.999 {
            vec3!(0.0, 0.0, 1.0)
        } else {
            vec3!(1.0, 0.0, 0.0)
        };
        let tangent = up.cross(normal).normalize();
        let bitangent = normal.cross(&tangent);
        let direction = tangent * h.x + bitangent * h.y + normal * h.z;
        direction.normalize()
    }

    fn geometry_smith(
        normal: &math::Vec3,
        view_direction: &math::Vec3,
        light_direction: &math::Vec3,
        roughness: f32,
    ) -> f32 {
        let roughness = roughness.powi(2) / 2.0;
        Self::geometry_ggx(normal, view_direction, roughness)
            * Self::geometry_ggx(normal, light_direction, roughness)
    }

    fn geometry_ggx(normal: &math::Vec3, direction: &math::Vec3, roughness: f32) -> f32 {
        let alignment = normal.dot(direction).max(0.0);
        alignment / (alignment * (1.0 - roughness) + roughness)
    }

    fn to_image(pixels: &[math::Vec3], width: u32, height: u32) -> Image {
        let mut image = image::RgbImage::new(width, height);
        for y in 0..height {
            let offset = (y * width) as usize;
            for x in 0..width {
                let pixel = &pixels[x as usize + offset];
                let pixel = [pixel.x as u8, pixel.y as u8, pixel.z as u8];
                image.put_pixel(x, y, image::Rgb(pixel))
            }
        }
        Image {
            width,
            height,
            data: image.into_raw(),
        }
    }

    fn to_hdr_image(pixels: &[math::Vec3], width: u32, height: u32) -> Image<f32> {
        Image {
            width,
            height,
            data: pixels.iter().flat_map(|p| [p.x, p.y, p.z]).collect(),
        }
    }

    fn bake_specular(&self, image: Arc<image::DynamicImage>) -> Vec<Image<f32>> {
        let image = Arc::new(image);
        let max_mip_level = u32::ilog2(image.width().max(image.height())) + 1;
        tracing::info!("Baking specular component with {max_mip_level} mip map levels");
        let image = image.clone();
        let brdf_sample_count = self.brdf_sample_count;
        (0..max_mip_level)
            .map(|level| {
                let minification = u32::pow(2, level);
                let image = image.resize(
                    image.width() / minification,
                    image.height() / minification,
                    image::imageops::FilterType::Lanczos3,
                );
                tracing::info!("Baking Mip Level {level}");
                let roughness = level as f32 / max_mip_level as f32;
                let specular_map =
                    Self::bake_specular_with_roughness(&image, roughness, brdf_sample_count);
                tracing::info!("Finished baking specular component ({level})");
                Self::to_hdr_image(&specular_map, image.width(), image.height())
            })
            .collect()
    }

    fn bake_specular_with_roughness(
        image: &image::DynamicImage,
        roughness: f32,
        sample_count: u32,
    ) -> Vec<math::Vec3> {
        let mut specular_map = Vec::new();
        for (x, y, _) in image.pixels() {
            let uv = UV::from_screen_coordinates(x, y, image.width(), image.height());
            let normal = uv.to_spheric();
            let normal = normal.to_cartesian().normalize();
            let specular = Self::calculate_specular(image, &normal, roughness, sample_count);
            specular_map.push(specular);
        }
        specular_map
    }

    fn calculate_specular(
        image: &image::DynamicImage,
        normal: &math::Vec3,
        roughness: f32,
        sample_count: u32,
    ) -> math::Vec3 {
        let mut total_weight = 0.0;
        let mut specular = vec3!(0.0);
        for i in 0..sample_count {
            let xi = Self::hammersley(i, sample_count);
            let h = Self::importance_sampling_ggx(xi, normal, roughness);
            let l = &(&h * 2.0 * normal.dot(&h)) - normal;
            let l = l.normalize();
            let nl = normal.dot(&l).max(0.0);
            if nl > 0.0 {
                specular = specular + (Self::sample(image, &l) * nl);
                total_weight += nl;
            }
        }
        specular / total_weight
    }

    fn radical_inverse_van_der_corput(bits: u32) -> f32 {
        let bits = bits.rotate_right(16u32);
        let bits = ((bits & 0x55555555u32) << 1u32) | ((bits & 0xAAAAAAAAu32) >> 1u32);
        let bits = ((bits & 0x33333333u32) << 2u32) | ((bits & 0xCCCCCCCCu32) >> 2u32);
        let bits = ((bits & 0x0F0F0F0Fu32) << 4u32) | ((bits & 0xF0F0F0F0u32) >> 4u32);
        let bits = ((bits & 0x00FF00FFu32) << 8u32) | ((bits & 0xFF00FF00u32) >> 8u32);
        bits as f32 * 2.328_306_4e-10_f32
    }

    fn bake_diffuse(&self, image: Arc<image::DynamicImage>, cores: usize) -> Image<f32> {
        let _ = tracing::info_span!("Diffuse");
        tracing::info!("Bake diffuse component with {cores} core(s)");
        let image_size = image.width() * image.height();
        let offset = image_size as usize / cores;
        let mut handles = Vec::new();
        for i in 0..cores {
            let handle =
                Self::bake_diffuse_async(image.clone(), offset * i, self.convolute_sample_delta);
            handles.push(handle);
        }
        let mut irradiance_map = Vec::new();
        for handle in handles {
            let mut result = handle.join().unwrap();
            irradiance_map.append(&mut result);
        }
        tracing::info!("Finished baking diffuse map");
        Self::to_hdr_image(&irradiance_map, image.width(), image.height())
    }

    fn bake_diffuse_async(
        image: Arc<image::DynamicImage>,
        offset: usize,
        sample_delta: f32,
    ) -> thread::JoinHandle<Vec<math::Vec3>> {
        thread::spawn(move || {
            let mut irradiance_map = Vec::new();
            for (x, y, _) in image.pixels().skip(offset) {
                let texcoord = UV::from_screen_coordinates(x, y, image.width(), image.height());
                let normal: Spheric = texcoord.to_spheric();
                let normal: math::Vec3 = normal.to_cartesian();
                let irradiance = Self::convolute(normal.normalize(), sample_delta, &image);
                irradiance_map.push(irradiance);
            }
            irradiance_map
        })
    }

    fn convolute(normal: math::Vec3, sample_delta: f32, image: &image::DynamicImage) -> math::Vec3 {
        let up = vec3!(0.0, 1.0, 0.0);
        let right = up.cross(&normal).normalize();
        let up = normal.cross(&right).normalize();
        let mut irradiance = vec3!(0.0);
        let mut sample_count: u32 = 0;
        let mut azimuth = 0.0;
        loop {
            if azimuth >= 2.0 * PI {
                break;
            }
            let mut inclination = 0.0;
            loop {
                if inclination >= 0.5 * PI {
                    break;
                }
                let direction = Spheric {
                    azimuth: inclination,
                    inclination: azimuth,
                };
                let direction = direction.to_cartesian();
                let direction =
                    (&right * direction.x) + (&up * direction.y) + (&normal * direction.z);
                let pixel = Self::sample(image, &direction);
                irradiance = irradiance + (pixel * inclination.cos() * inclination.sin());
                sample_count += 1;
                inclination += sample_delta;
            }
            azimuth += sample_delta;
        }
        irradiance / sample_count as f32
    }

    fn sample(image: &image::DynamicImage, direction: &math::Vec3) -> math::Vec3 {
        let spherical = Spheric::from_cartesian(direction);
        let texcoord = UV::from_spheric(&spherical);
        let texcoord = texcoord.to_screen_coordinates(image.width(), image.height());
        let image = match image {
            image::DynamicImage::ImageRgb32F(image) => image,
            _ => panic!("non-hdr image supplied"),
        };
        let pixel = image.get_pixel(texcoord.x as u32, texcoord.y as u32).0;
        vec3!(pixel[0], pixel[1], pixel[2])
    }
}
