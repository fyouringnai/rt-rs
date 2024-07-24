use crate::utils::{MATERIAL, SHAPE};

#[derive(Clone)]
pub struct Object {
    pub shape: SHAPE,
    pub vertices: Vec<[f32; 3]>,
    pub center: [f32; 3],
    pub radius: f32,
    pub albedo: [f32; 3],
    pub material: MATERIAL,
}

impl Object {
    pub fn new_sphere(
        center: [f32; 3],
        radius: f32,
        albedo: [f32; 3],
        material: MATERIAL,
    ) -> Object {
        Object {
            shape: SHAPE::RT_SPHERE,
            vertices: Vec::new(),
            center,
            radius,
            albedo,
            material,
        }
    }

    pub fn new_mesh(vertices: Vec<[f32; 3]>, material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_MESH,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo: [0.0, 0.0, 0.0],
            material,
        }
    }

    pub fn new_triangle(vertices: Vec<[f32; 3]>, albedo: [f32; 3], material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_TRIANGLE,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo,
            material,
        }
    }

    pub fn new_rectangle(vertices: Vec<[f32; 3]>, albedo: [f32; 3], material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_RECTANGLE,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo,
            material,
        }
    }

}
