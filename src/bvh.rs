use crate::aabb::{aabb_axis, merge_aabb, merge_vec3, AABB};
use crate::object::Object;
use crate::shader::Shader;
use crate::utils::SHAPE;
use bytemuck::cast_slice;
use glow::{
    Context, HasContext, Texture, CLAMP_TO_EDGE, FLOAT, NEAREST, NO_ERROR, RGB, RGB32F, TEXTURE1,
    TEXTURE2, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T,
};
use std::cell::RefCell;
use std::rc::Rc;

pub struct BVHNode {
    pub aabb: AABB,
    pub left: Option<Rc<RefCell<BVHNode>>>,
    pub right: Option<Rc<RefCell<BVHNode>>>,
    pub primitive_number: i32,
    pub first_offset: i32,
    pub axis: i32,
}

impl BVHNode {
    pub fn new() -> BVHNode {
        BVHNode {
            aabb: AABB::new(),
            left: None,
            right: None,
            primitive_number: 0,
            first_offset: 0,
            axis: 0,
        }
    }
    pub fn new_leaf(primitive_number: i32, first_offset: i32, aabb: AABB) -> BVHNode {
        BVHNode {
            aabb,
            left: None,
            right: None,
            primitive_number,
            first_offset,
            axis: 0,
        }
    }
    pub fn new_interior(
        left: Option<Rc<RefCell<BVHNode>>>,
        right: Option<Rc<RefCell<BVHNode>>>,
        aabb: AABB,
        axis: i32,
    ) -> BVHNode {
        BVHNode {
            aabb,
            left,
            right,
            primitive_number: 0,
            first_offset: 0,
            axis,
        }
    }
}

pub struct LinearBVHNode {
    pub aabb: AABB,
    pub offset: i32,
    pub n_primitives: i32,
    pub axis: i32,
}

impl LinearBVHNode {
    pub fn new(aabb: AABB, offset: i32, n_primitives: i32, axis: i32) -> LinearBVHNode {
        LinearBVHNode {
            aabb,
            offset,
            n_primitives,
            axis,
        }
    }
}

struct BVHPrimitiveInfo {
    pub primitive_number: i32,
    pub centroid: [f32; 3],
    pub aabb: AABB,
}

impl BVHPrimitiveInfo {
    pub fn new(primitive_number: i32, bounds: AABB) -> BVHPrimitiveInfo {
        let centroid = [
            (bounds.min[0] + bounds.max[0]) * 0.5,
            (bounds.min[1] + bounds.max[1]) * 0.5,
            (bounds.min[2] + bounds.max[2]) * 0.5,
        ];
        BVHPrimitiveInfo {
            primitive_number,
            centroid,
            aabb: bounds,
        }
    }
}

pub struct BVHTree {
    primitives: Vec<Object>,
    linear_bvh_node: Vec<LinearBVHNode>,
    bvh_texture: Texture,
    vertices_texture: Texture,
    node_number: i32,
    vertices_number: i32,
}

impl BVHTree {
    pub fn new(gl: &Context) -> BVHTree {
        let bvh_tree = BVHTree {
            primitives: Vec::new(),
            linear_bvh_node: Vec::new(),
            bvh_texture: unsafe { gl.create_texture().unwrap() },
            vertices_texture: unsafe { gl.create_texture().unwrap() },
            node_number: 0,
            vertices_number: 0,
        };
        bvh_tree
    }

    pub fn build(&mut self, primitives: &Vec<Object>) {
        self.primitives = primitives.clone();
        let mut primitive_info = Vec::new();
        for (i, primitive) in self.primitives.iter().enumerate() {
            let mut aabb = AABB::new();
            match primitive.shape {
                SHAPE::NONE => {}
                SHAPE::RT_SPHERE => {
                    aabb = AABB::new_sphere(
                        primitive.center,
                        primitive.radius,
                        primitive.constant,
                        primitive.material.clone(),
                    );
                }
                SHAPE::RT_MESH => {
                    let mut vertex = Vec::new();
                    for i in 0..3 {
                        vertex.push(primitive.vertices[i * 3]);
                    }
                    aabb = AABB::new_mesh(vertex, primitive.constant, primitive.material.clone());
                }
                SHAPE::RT_TRIANGLE => {
                    let mut vertex = Vec::new();
                    for i in 0..3 {
                        vertex.push(primitive.vertices[i]);
                    }
                    aabb =
                        AABB::new_triangle(vertex, primitive.constant, primitive.material.clone());
                }
                SHAPE::RT_RECTANGLE => {
                    let mut vertex = Vec::new();
                    for i in 0..4 {
                        vertex.push(primitive.vertices[i]);
                    }
                    aabb =
                        AABB::new_rectangle(vertex, primitive.constant, primitive.material.clone());
                }
                SHAPE::RT_VOLUME => {
                    let mut vertex = Vec::new();
                    for i in 0..4 {
                        vertex.push(primitive.vertices[i]);
                    }
                    aabb = AABB::new_box_volume(
                        vertex,
                        primitive.constant,
                        primitive.material.clone(),
                    );
                }
            }

            primitive_info.push(BVHPrimitiveInfo::new(i as i32, aabb));
        }
        let mut ordered_prims = Vec::new();
        let end = self.primitives.len() as i32;
        let mut total_nodes = 0;
        let root = self.recursive_build(
            &mut primitive_info,
            0,
            end,
            &mut total_nodes,
            &mut ordered_prims,
        );
        self.primitives = ordered_prims;

        let mut offset = 0;
        self.flatten_bvh(&root, &mut offset);
    }

    pub fn set_texture(&mut self, gl: &Context) {
        let mut node_data = Vec::new();
        for node in &self.linear_bvh_node {
            node_data.push(node.aabb.min[0]);
            node_data.push(node.aabb.min[1]);
            node_data.push(node.aabb.min[2]);
            node_data.push(node.aabb.max[0]);
            node_data.push(node.aabb.max[1]);
            node_data.push(node.aabb.max[2]);
            node_data.push(node.offset as f32);
            node_data.push(node.n_primitives as f32);
            node_data.push(node.axis as f32);
            node_data.push(node.aabb.shape.clone() as u32 as f32);
            node_data.push(node.aabb.constant);
            node_data.push(node.aabb.material.clone() as u32 as f32);
            self.node_number += 1;
        }
        let mut vertex_data = Vec::new();
        for primitive in &self.primitives {
            match primitive.shape {
                SHAPE::NONE => {}
                SHAPE::RT_SPHERE => {
                    for i in 0..3 {
                        vertex_data.push(primitive.center[i]);
                    }
                    for i in 0..3 {
                        vertex_data.push(primitive.albedo[i]);
                    }
                    vertex_data.push(primitive.radius);
                    for _i in 0..26 {
                        vertex_data.push(0.0);
                    }
                    self.vertices_number += 1;
                }
                SHAPE::RT_MESH => {
                    for vertex in &primitive.vertices {
                        vertex_data.push(vertex[0]);
                        vertex_data.push(vertex[1]);
                        vertex_data.push(vertex[2]);
                    }
                    self.vertices_number += 1;
                }
                SHAPE::RT_TRIANGLE => {
                    for vertex in &primitive.vertices {
                        vertex_data.push(vertex[0]);
                        vertex_data.push(vertex[1]);
                        vertex_data.push(vertex[2]);
                    }
                    for i in 0..3 {
                        vertex_data.push(primitive.albedo[i]);
                    }
                    for _i in 0..18 {
                        vertex_data.push(0.0);
                    }
                    self.vertices_number += 1;
                }
                SHAPE::RT_RECTANGLE => {
                    for vertex in &primitive.vertices {
                        vertex_data.push(vertex[0]);
                        vertex_data.push(vertex[1]);
                        vertex_data.push(vertex[2]);
                    }
                    for i in 0..3 {
                        vertex_data.push(primitive.albedo[i]);
                    }
                    for _i in 0..15 {
                        vertex_data.push(0.0);
                    }
                    self.vertices_number += 1;
                }
                SHAPE::RT_VOLUME => {
                    for vertex in &primitive.vertices {
                        vertex_data.push(vertex[0]);
                        vertex_data.push(vertex[1]);
                        vertex_data.push(vertex[2]);
                    }
                    for i in 0..3 {
                        vertex_data.push(primitive.albedo[i]);
                    }
                    for _i in 0..18 {
                        vertex_data.push(0.0);
                    }
                    self.vertices_number += 1;
                }
            }
        }
        let bvh_texture_size = self.node_number * 4;
        let vertex_texture_size = self.vertices_number * 11;
        unsafe {
            gl.bind_texture(TEXTURE_2D, Some(self.bvh_texture));
            assert_eq!(gl.get_error(), NO_ERROR);
            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGB32F as i32,
                get_length(bvh_texture_size),
                get_length(bvh_texture_size),
                0,
                RGB,
                FLOAT,
                Some(cast_slice(&node_data)),
            );
            assert_eq!(gl.get_error(), NO_ERROR);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

            gl.bind_texture(TEXTURE_2D, Some(self.vertices_texture));
            assert_eq!(gl.get_error(), NO_ERROR);
            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGB32F as i32,
                get_length(vertex_texture_size),
                get_length(vertex_texture_size),
                0,
                RGB,
                FLOAT,
                Some(cast_slice(&vertex_data)),
            );
            assert_eq!(gl.get_error(), NO_ERROR);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);
        }
    }

    pub fn use_texture(&self, gl: &Context, shader: &Shader) {
        unsafe {
            shader.use_program(gl);
            gl.active_texture(TEXTURE1);
            gl.bind_texture(TEXTURE_2D, Some(self.vertices_texture));
            shader.set_int(&gl, "vertices_texture", 1);
            gl.active_texture(TEXTURE2);
            shader.set_int(gl, "bvh_texture", 2);
            gl.bind_texture(TEXTURE_2D, Some(self.bvh_texture));
            assert_eq!(gl.get_error(), NO_ERROR);
        }
    }

    pub fn delete_texture(&self, gl: &Context) {
        unsafe {
            gl.delete_texture(self.bvh_texture);
            gl.delete_texture(self.vertices_texture);
        }
    }

    pub fn create_texture(&mut self, gl: &Context) {
        self.bvh_texture = unsafe { gl.create_texture().unwrap() };
        self.vertices_texture = unsafe { gl.create_texture().unwrap() };
    }

    fn recursive_build(
        &self,
        primitive_info: &mut Vec<BVHPrimitiveInfo>,
        start: i32,
        end: i32,
        total_nodes: &mut i32,
        ordered_primitives: &mut Vec<Object>,
    ) -> BVHNode {
        #[allow(unused_assignments)]
        let mut node = BVHNode::new();
        let mut aabb = AABB::new();
        for i in start..end {
            aabb = merge_aabb(&aabb, &primitive_info[i as usize].aabb);
        }
        let primitives_number = end - start;
        if primitives_number == 1 {
            let first_offset = ordered_primitives.len() as i32;
            #[allow(unused_assignments)]
            let mut primitive_number = 0;
            for i in start..end {
                primitive_number = primitive_info[i as usize].primitive_number;
                ordered_primitives.push(self.primitives[primitive_number as usize].clone());
            }
            *total_nodes += 1;
            node = BVHNode::new_leaf(
                primitives_number,
                first_offset,
                primitive_info[start as usize].aabb.clone(),
            );
            node
        } else {
            let mut centroid_aabb = AABB::new();
            for i in start..end {
                centroid_aabb = merge_vec3(&centroid_aabb, &primitive_info[i as usize].centroid);
            }
            let dim = aabb_axis(&centroid_aabb);

            let mid = (start + end) / 2;

            partition_by_median(primitive_info, start, mid, end, dim);

            *total_nodes += 1;
            node = BVHNode::new_interior(
                Some(Rc::new(RefCell::from(self.recursive_build(
                    primitive_info,
                    start,
                    mid,
                    total_nodes,
                    ordered_primitives,
                )))),
                Some(Rc::new(RefCell::from(self.recursive_build(
                    primitive_info,
                    mid,
                    end,
                    total_nodes,
                    ordered_primitives,
                )))),
                aabb,
                dim,
            );
            node
        }
    }

    fn flatten_bvh(&mut self, node: &BVHNode, offset: &mut i32) -> i32 {
        let my_offset = *offset;
        *offset += 1;
        let linear_node =
            LinearBVHNode::new(node.aabb.clone(), 0, node.primitive_number, node.axis);

        self.linear_bvh_node.push(linear_node);

        if node.primitive_number > 0 {
            self.linear_bvh_node[my_offset as usize].offset = node.first_offset;
            self.linear_bvh_node[my_offset as usize].n_primitives = node.primitive_number;
        } else {
            self.linear_bvh_node[my_offset as usize].axis = node.axis;
            self.linear_bvh_node[my_offset as usize].n_primitives = 0;
            self.flatten_bvh(&node.left.as_ref().unwrap().borrow(), offset);
            self.linear_bvh_node[my_offset as usize].offset =
                self.flatten_bvh(&node.right.as_ref().unwrap().borrow(), offset);
        }
        my_offset
    }
}

fn partition_by_median(
    primitive_info: &mut Vec<BVHPrimitiveInfo>,
    start: i32,
    mid: i32,
    end: i32,
    dim: i32,
) {
    let mut left = start;
    let mut right = end - 1;
    loop {
        while left < right
            && primitive_info[left as usize].centroid[dim as usize]
                < primitive_info[mid as usize].centroid[dim as usize]
        {
            left += 1;
        }
        while left < right
            && primitive_info[right as usize].centroid[dim as usize]
                >= primitive_info[mid as usize].centroid[dim as usize]
        {
            right -= 1;
        }
        if left >= right {
            break;
        }
        primitive_info.swap(left as usize, right as usize);
    }
}

fn get_length(size: i32) -> i32 {
    let mut length = 1;
    let edge = (size as f32).sqrt().ceil() as i32;
    while length < edge {
        length <<= 1;
    }
    length
}
