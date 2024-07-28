use crate::utils::MATERIAL::NONE;
use crate::utils::SHAPE::{RT_MESH, RT_RECTANGLE, RT_SPHERE};
use crate::utils::{MATERIAL, MAX_FLOAT, MIN_FLOAT, SHAPE};

#[derive(Clone)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub shape: SHAPE,
    pub constant: f32,
    pub material: MATERIAL,
}

impl AABB {
    pub fn new() -> AABB {
        let aabb = AABB {
            min: [MAX_FLOAT, MAX_FLOAT, MAX_FLOAT],
            max: [MIN_FLOAT, MIN_FLOAT, MIN_FLOAT],
            shape: SHAPE::NONE,
            constant: 0.0,
            material: NONE,
        };

        aabb
    }

    pub fn new_mesh(vertices: Vec<[f32; 3]>, constant: f32, material: MATERIAL) -> AABB {
        let mut aabb = AABB {
            min: [MAX_FLOAT, MAX_FLOAT, MAX_FLOAT],
            max: [MIN_FLOAT, MIN_FLOAT, MIN_FLOAT],
            shape: RT_MESH,
            constant,
            material,
        };

        for vertex in vertices {
            for i in 0..3 {
                if vertex[i] < aabb.min[i] {
                    aabb.min[i] = vertex[i];
                }
                if vertex[i] > aabb.max[i] {
                    aabb.max[i] = vertex[i];
                }
            }
        }

        aabb
    }

    pub fn new_sphere(center: [f32; 3], radius: f32, constant: f32, material: MATERIAL) -> AABB {
        let aabb = AABB {
            min: [center[0] - radius, center[1] - radius, center[2] - radius],
            max: [center[0] + radius, center[1] + radius, center[2] + radius],
            shape: RT_SPHERE,
            constant,
            material,
        };

        aabb
    }

    pub fn new_triangle(vertices: Vec<[f32; 3]>, constant: f32, material: MATERIAL) -> AABB {
        let mut aabb = AABB {
            min: [MAX_FLOAT, MAX_FLOAT, MAX_FLOAT],
            max: [MIN_FLOAT, MIN_FLOAT, MIN_FLOAT],
            shape: SHAPE::RT_TRIANGLE,
            constant,
            material,
        };

        for vertex in vertices {
            for i in 0..3 {
                if vertex[i] < aabb.min[i] {
                    aabb.min[i] = vertex[i];
                }
                if vertex[i] > aabb.max[i] {
                    aabb.max[i] = vertex[i];
                }
            }
        }

        aabb
    }

    pub fn new_rectangle(vertices: Vec<[f32; 3]>, constant: f32, material: MATERIAL) -> AABB {
        let mut aabb = AABB {
            min: [MAX_FLOAT, MAX_FLOAT, MAX_FLOAT],
            max: [MIN_FLOAT, MIN_FLOAT, MIN_FLOAT],
            shape: RT_RECTANGLE,
            constant,
            material,
        };

        for vertex in vertices {
            for i in 0..3 {
                if vertex[i] < aabb.min[i] {
                    aabb.min[i] = vertex[i];
                }
                if vertex[i] > aabb.max[i] {
                    aabb.max[i] = vertex[i];
                }
            }
        }

        aabb
    }
}

pub fn merge_aabb(a: &AABB, b: &AABB) -> AABB {
    let mut aabb = AABB::new();

    for i in 0..3 {
        aabb.min[i] = a.min[i].min(b.min[i]);
        aabb.max[i] = a.max[i].max(b.max[i]);
    }

    aabb
}

pub fn merge_vec3(a: &AABB, b: &[f32; 3]) -> AABB {
    let mut aabb = AABB::new();

    for i in 0..3 {
        aabb.min[i] = a.min[i].min(b[i]);
        aabb.max[i] = a.max[i].max(b[i]);
    }

    aabb
}

pub fn aabb_axis(a: &AABB) -> i32 {
    let d = [
        a.max[0] - a.min[0],
        a.max[1] - a.min[1],
        a.max[2] - a.min[2],
    ];
    if d[0] > d[1] && d[0] > d[2] {
        0
    } else if d[1] > d[2] {
        1
    } else {
        2
    }
}
