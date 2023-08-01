use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

use crate::{
    model::{Material, Vertex},
    window::Framebuffer,
};

fn project(p: Vec3, mvp: Mat4) -> (Vec3, f32) {
    let proj_pos = mvp * Vec4::from((p, 1.0));
    let rec = 1.0 / proj_pos.w;
    let rec_pos = proj_pos * rec;
    (Vec3::new(rec_pos.x, rec_pos.y, rec_pos.z), rec)
}

fn clip_to_screen_space(clip_space: Vec2, screen_size: Vec2) -> Vec2 {
    (clip_space * -0.5 + 0.5) * screen_size
}

fn edge_function(a: Vec2, c: Vec2, b: Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

pub struct Triangle {
    pub vertex: [Vertex; 3],
}

impl Triangle {
    pub fn draw(
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
