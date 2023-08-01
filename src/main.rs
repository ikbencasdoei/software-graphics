#![feature(slice_as_chunks)]

use std::time::SystemTime;

use glam::*;

mod window;
use window::{Framebuffer, Window};
mod model;
use model::Model;
mod texture;
use texture::Texture;

mod triangle;

fn main() {
    let mut window = Window::new(env!("CARGO_PKG_NAME"), 512, 512);
    let mut depth_buffer =
        Framebuffer::new(window.framebuffer().width(), window.framebuffer().height());

    let model = Model::load("assets/DamagedHelmet/DamagedHelmet.gltf");

    let timer = SystemTime::now();

    while !window.should_close() {
        let framebuffer = window.framebuffer();

        if framebuffer.width() != depth_buffer.width()
            || framebuffer.height() != depth_buffer.height()
        {
            depth_buffer = Framebuffer::new(framebuffer.width(), framebuffer.height());
        }

        framebuffer.clear(Vec3::splat(0.1));
        depth_buffer.clear(1.0);

        let aspect_ratio = framebuffer.width() as f32 / framebuffer.height() as f32;
        let model_matrix =
            Mat4::from_axis_angle(
                Vec3::new(0.0, 1.0, 0.0),
                timer.elapsed().unwrap().as_secs_f32(),
            ) * Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), (90.0f32).to_radians());
        let view_matrix = Mat4::from_translation(Vec3::new(0.0, 0.0, -2.5));
        let proj_matrix = Mat4::perspective_rh((60.0f32).to_radians(), aspect_ratio, 0.01, 300.0);
        let mvp_matrix = proj_matrix * view_matrix * model_matrix;
        let inv_trans_model_matrix = model_matrix.inverse().transpose();

        model.draw(
            framebuffer,
            &mut depth_buffer,
            mvp_matrix,
            inv_trans_model_matrix,
        );

        window.display();
    }
}
