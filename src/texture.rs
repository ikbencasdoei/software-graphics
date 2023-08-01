use glam::*;

#[derive(Clone, Debug)]
pub struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

pub fn load_texture(file_path: &str) -> Texture {
    let image = image::open(file_path).unwrap();
    let rgba = image.into_rgba8();

    Texture {
        width: rgba.width(),
        height: rgba.height(),
        data: rgba.into_raw(),
    }
}

impl Texture {
    pub fn sample_pixel(&self, x: f32, y: f32) -> Vec4 {
        let inv_dims = Vec2::new(1.0 / self.width as f32, 1.0 / self.height as f32);

        let tl = self.get_pixel(x - inv_dims.x, y - inv_dims.y);
        let bl = self.get_pixel(x - inv_dims.x, y + inv_dims.y);
        let br = self.get_pixel(x + inv_dims.x, y + inv_dims.y);
        let tr = self.get_pixel(x + inv_dims.x, y - inv_dims.y);

        let x = x * self.width as f32;
        let y = y * self.height as f32;
        let dx = x - ((x as i32) as f32);
        let dy = y - ((y as i32) as f32);

        let bottom = bl.lerp(br, dx);
        let top = tl.lerp(tr, dx);
        top.lerp(bottom, dy)
    }

    pub fn get_pixel(&self, x: f32, y: f32) -> Vec4 {
        let x = ((x * self.width as f32) as usize) % (self.width - 1) as usize;
        let y = ((y * self.height as f32) as usize) % (self.height - 1) as usize;

        let data: &Vec<(u8, u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
        let pixel = &data[y * (self.width as usize) + x];

        Vec4::new(
            pixel.0 as f32 / 255.99,
            pixel.1 as f32 / 255.99,
            pixel.2 as f32 / 255.99,
            pixel.3 as f32 / 255.99,
        )
    }
}
