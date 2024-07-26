#![allow(non_camel_case_types)]

use cgmath::{vec4, Deg, Matrix, Matrix4, SquareMatrix, Vector3};
use rand::Rng;

#[derive(Clone, PartialEq)]
pub enum SHAPE {
    NONE = 0,
    RT_SPHERE = 1,
    RT_MESH = 2,
    RT_TRIANGLE = 3,
    RT_RECTANGLE = 4,
}

#[derive(Clone)]
pub enum MATERIAL {
    NONE = 0,
    DIFFUSE = 1,
    METAL = 2,
    DIELECTRIC = 3,
    DIFFUSE_LIGHT = 4,
}

pub const MAX_FLOAT: f32 = 3.402823466e+38;
pub const MIN_FLOAT: f32 = -3.402823466e+38;

pub fn random_float() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}

pub fn translated(v: &[f32; 3], model: &Matrix4<f32>) -> [f32; 3] {
    let translated = model * vec4(v[0], v[1], v[2], 1.0);
    [
        translated.x / translated.w,
        translated.y / translated.w,
        translated.z / translated.w,
    ]
}

pub fn translated_normal(v: &[f32; 3], model: &Matrix4<f32>) -> [f32; 3] {
    let translated = model.invert().unwrap().transpose() * vec4(v[0], v[1], v[2], 0.0);
    [translated.x, translated.y, translated.z]
}

pub fn trans(
    translation: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
) -> Matrix4<f32> {
    let mut model = Matrix4::<f32>::identity();
    model = model * Matrix4::from_translation(translation);
    model = model * Matrix4::from_angle_x(Deg(rotation.x));
    model = model * Matrix4::from_angle_y(Deg(rotation.y));
    model = model * Matrix4::from_angle_z(Deg(rotation.z));
    model = model * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
    model
}
