use crate::{clip_to_screen_space, edge_function, project, window::Framebuffer, Texture};
use glam::*;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vec3::ZERO,
            normal: Vec3::ZERO,
            tex_coord: Vec2::ZERO,
        }
    }
}

struct Triangle {
    vertex: [Vertex; 3],
}

impl Triangle {
    fn draw(
        &self,
        framebuffer: &mut Framebuffer<Vec3>,
        depth_buffer: &mut Framebuffer<f32>,
        mvp: Mat4,
        inv_trans_model_matrix: Mat4,
        material: &Material,
    ) {
        let v0 = self.vertex[0];
        let v1 = self.vertex[1];
        let v2 = self.vertex[2];

        let v0_clip_space = project(v0.position, mvp);
        let v1_clip_space = project(v1.position, mvp);
        let v2_clip_space = project(v2.position, mvp);

        let screen_size = Vec2::new(framebuffer.width() as f32, framebuffer.height() as f32);
        let v0_screen_space = clip_to_screen_space(v0_clip_space.0.xy(), screen_size);
        let v1_screen_space = clip_to_screen_space(v1_clip_space.0.xy(), screen_size);
        let v2_screen_space = clip_to_screen_space(v2_clip_space.0.xy(), screen_size);

        let min = v0_screen_space
            .min(v1_screen_space.min(v2_screen_space))
            .max(Vec2::ZERO);
        let max =
            (v0_screen_space.max(v1_screen_space.max(v2_screen_space)) + 1.0).min(screen_size);

        for x in (min.x as usize)..(max.x as usize) {
            for y in (min.y as usize)..(max.y as usize) {
                let p = Vec2::new(x as f32, y as f32) + 0.5;

                let a0 = edge_function(v1_screen_space, v2_screen_space, p);
                let a1 = edge_function(v2_screen_space, v0_screen_space, p);
                let a2 = edge_function(v0_screen_space, v1_screen_space, p);
                let overlaps = a0 > 0.0 && a1 > 0.0 && a2 > 0.0;

                if overlaps {
                    let area_rep =
                        1.0 / edge_function(v0_screen_space, v1_screen_space, v2_screen_space);
                    let bary_coords = Vec3::new(a0, a1, a2) * area_rep;
                    let correction = 1.0
                        / (bary_coords.x * v0_clip_space.1
                            + bary_coords.y * v1_clip_space.1
                            + bary_coords.z * v2_clip_space.1);

                    let z = v0_clip_space.0.z * bary_coords.x
                        + v1_clip_space.0.z * bary_coords.y
                        + v2_clip_space.0.z * bary_coords.z;
                    let depth = depth_buffer.get_pixel(x, y);

                    if z < depth {
                        depth_buffer.set_pixel(x, y, z);

                        let n0 = inv_trans_model_matrix * Vec4::from((v0.normal, 1.0));
                        let n1 = inv_trans_model_matrix * Vec4::from((v1.normal, 1.0));
                        let n2 = inv_trans_model_matrix * Vec4::from((v2.normal, 1.0));
                        let normal = ((n0 * v0_clip_space.1 * bary_coords.x
                            + n1 * v1_clip_space.1 * bary_coords.y
                            + n2 * v2_clip_space.1 * bary_coords.z)
                            .xyz()
                            * correction)
                            .normalize();

                        let tex_coord = (v0.tex_coord * v0_clip_space.1 * bary_coords.x
                            + v1.tex_coord * v1_clip_space.1 * bary_coords.y
                            + v2.tex_coord * v2_clip_space.1 * bary_coords.z)
                            * correction;

                        let mut base_color = material.base_color;
                        if let Some(base_color_texture) = &material.base_color_texture {
                            base_color *= base_color_texture.sample_pixel(tex_coord.x, tex_coord.y);
                        }

                        let light_dir = Vec3::new(0.3, -0.8, -0.4).normalize();
                        let light_intensity = normal.dot(-light_dir);

                        let final_color = base_color * light_intensity;

                        framebuffer.set_pixel(x, y, final_color.xyz());
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_idx: usize,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load(file_path: &str) -> Self {
        let (document, buffers, _images) = gltf::import(file_path).expect("Failed to load model.");

        let mut meshes = Vec::new();
        let mut materials = vec![Material::default(); document.materials().len()];
        if materials.is_empty() {
            materials.push(Material::default());
        }

        if document.nodes().len() > 0 {
            process_node(
                document.nodes().next().as_ref().unwrap(),
                &buffers,
                &mut meshes,
                &mut materials,
                file_path,
            );
        }

        Self { meshes, materials }
    }

    pub fn draw(
        &self,
        framebuffer: &mut Framebuffer<Vec3>,
        depth_buffer: &mut Framebuffer<f32>,
        mvp: Mat4,
        inv_trans_model_matrix: Mat4,
    ) {
        for mesh in &self.meshes {
            for i in 0..(mesh.indices.len() / 3) {
                let v0 = mesh.vertices[mesh.indices[i * 3] as usize];
                let v1 = mesh.vertices[mesh.indices[i * 3 + 1] as usize];
                let v2 = mesh.vertices[mesh.indices[i * 3 + 2] as usize];

                let triangle = Triangle {
                    vertex: [v0, v1, v2],
                };

                let material = &self.materials[mesh.material_idx];

                triangle.draw(
                    framebuffer,
                    depth_buffer,
                    mvp,
                    inv_trans_model_matrix,
                    material,
                );
            }
        }
    }
}

fn process_node(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    meshes: &mut Vec<Mesh>,
    materials: &mut [Material],
    file_path: &str,
) {
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.mode() == gltf::mesh::Mode::Triangles {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = {
                    let iter = reader
                        .read_positions()
                        .expect("Failed to process mesh node. (Vertices must have positions)");

                    iter.map(|arr| -> Vec3 { Vec3::from(arr) })
                        .collect::<Vec<_>>()
                };

                let mut vertices: Vec<Vertex> = positions
                    .into_iter()
                    .map(|position| Vertex {
                        position,
                        ..Default::default()
                    })
                    .collect();

                if let Some(normals) = reader.read_normals() {
                    for (i, normal) in normals.enumerate() {
                        vertices[i].normal = Vec3::from(normal);
                    }
                }

                if let Some(tex_coords) = reader.read_tex_coords(0) {
                    for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                        vertices[i].tex_coord = Vec2::from(tex_coord);
                    }
                }

                let indices = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
                    .expect("Failed to process mesh node. (Indices are required)");

                let prim_material = primitive.material();
                let pbr = prim_material.pbr_metallic_roughness();
                let material_idx = primitive.material().index().unwrap_or(0);

                let material = &mut materials[material_idx];
                material.base_color = Vec4::from(pbr.base_color_factor());
                if let Some(base_color_texture) = pbr.base_color_texture() {
                    if let gltf::image::Source::Uri { uri, .. } =
                        base_color_texture.texture().source().source()
                    {
                        let model_path = Path::new(file_path);
                        let texture_path = model_path
                            .parent()
                            .unwrap_or_else(|| Path::new("./"))
                            .join(uri);
                        let texture_path_str = texture_path.into_os_string().into_string().unwrap();

                        material.base_color_texture = Some(Texture::load(&texture_path_str));
                    }
                }

                meshes.push(Mesh {
                    vertices,
                    indices,
                    material_idx,
                });
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Material {
    pub base_color: Vec4,
    pub base_color_texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vec4::ONE,
            base_color_texture: None,
        }
    }
}
