use crate::utils::{MATERIAL, SHAPE};

#[derive(Clone)]
pub struct Object {
    pub shape: SHAPE,
    pub vertices: Vec<[f32; 3]>,
    pub center: [f32; 3],
    pub radius: f32,
    pub material: MATERIAL,
}

impl Object {
    pub fn new_sphere(center: [f32; 3], radius: f32, material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_SPHERE,
            vertices: Vec::new(),
            center,
            radius,
            material,
        }
    }

    pub fn new_triangle(vertices: Vec<[f32; 3]>, material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_TRIANGLE,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            material,
        }
    }
}
