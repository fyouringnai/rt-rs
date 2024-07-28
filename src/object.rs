use cgmath::Vector3;

use crate::utils::{trans, translated, translated_normal, MATERIAL, SHAPE};

#[derive(Clone)]
pub struct Object {
    pub shape: SHAPE,
    pub vertices: Vec<[f32; 3]>,
    pub center: [f32; 3],
    pub radius: f32,
    pub albedo: [f32; 3],
    pub constant: f32,
    pub material: MATERIAL,
}

impl Object {
    pub fn new_sphere(
        center: [f32; 3],
        radius: f32,
        albedo: [f32; 3],
        transform: &Vec<Vector3<f32>>,
        constant: f32,
        material: MATERIAL,
    ) -> Object {
        Object {
            shape: SHAPE::RT_SPHERE,
            vertices: Vec::new(),
            center: translated(&center, &trans(transform[0], transform[1], transform[2])),
            radius,
            albedo,
            constant,
            material,
        }
    }

    pub fn new_mesh(vertices: Vec<[f32; 3]>, constant: f32, material: MATERIAL) -> Object {
        Object {
            shape: SHAPE::RT_MESH,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo: [0.0, 0.0, 0.0],
            constant,
            material,
        }
    }

    pub fn new_triangle(
        vertices: &Vec<[f32; 3]>,
        albedo: [f32; 3],
        transform: &Vec<Vector3<f32>>,
        constant: f32,
        material: MATERIAL,
    ) -> Object {
        let model = trans(transform[0], transform[1], transform[2]);
        let vertices = vec![
            translated(&vertices[0], &model),
            translated(&vertices[1], &model),
            translated(&vertices[2], &model),
            translated_normal(&vertices[3], &model),
        ];
        Object {
            shape: SHAPE::RT_TRIANGLE,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo,
            constant,
            material,
        }
    }

    pub fn new_rectangle(
        vertices: &Vec<[f32; 3]>,
        albedo: [f32; 3],
        transform: &Vec<Vector3<f32>>,
        constant: f32,
        material: MATERIAL,
    ) -> Object {
        let model = trans(transform[0], transform[1], transform[2]);
        let vertices = vec![
            translated(&vertices[0], &model),
            translated(&vertices[1], &model),
            translated(&vertices[2], &model),
            translated(&vertices[3], &model),
            translated_normal(&vertices[4], &model),
        ];
        Object {
            shape: SHAPE::RT_RECTANGLE,
            vertices,
            center: [0.0, 0.0, 0.0],
            radius: 0.0,
            albedo,
            constant,
            material,
        }
    }

    pub fn new_box(
        vertices: &Vec<[f32; 3]>,
        albedo: [f32; 3],
        transform: &Vec<Vector3<f32>>,
        constant: f32,
        material: MATERIAL,
    ) -> Vec<Object> {
        let mut objects = Vec::new();

        let model = trans(transform[0], transform[1], transform[2]);
        for i in 0..6 {
            let vertex = vec![
                translated(&vertices[i * 5], &model),
                translated(&vertices[i * 5 + 1], &model),
                translated(&vertices[i * 5 + 2], &model),
                translated(&vertices[i * 5 + 3], &model),
                translated_normal(&vertices[i * 5 + 4], &model),
            ];
            let object = Object {
                shape: SHAPE::RT_RECTANGLE,
                vertices: vertex,
                center: [0.0, 0.0, 0.0],
                radius: 0.0,
                albedo,
                constant,
                material: material.clone(),
            };
            objects.push(object);
        }
        objects
    }
}
