#![allow(non_camel_case_types)]
use rand::Rng;

#[derive(Clone, PartialEq)]
pub enum SHAPE {
    NONE = 0,
    RT_SPHERE = 1,
    RT_TRIANGLE = 2,
}

#[derive(Clone)]
pub enum MATERIAL {
    NONE = 0,
    DIFFUSE = 1,
    METAL = 2,
    DIELECTRIC = 3,
}

pub const MAX_FLOAT: f32 = 3.402823466e+38;
pub const MIN_FLOAT: f32 = -3.402823466e+38;

pub fn random_float() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}
