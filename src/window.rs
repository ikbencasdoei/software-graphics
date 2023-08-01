use glam::Vec3;

pub struct Window {
    window: minifb::Window,
    framebuffer: Framebuffer<Vec3>,
}

impl Window {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        let options = minifb::WindowOptions {
            resize: true,
            ..Default::default()
        };

        let window =
            minifb::Window::new(name, width, height, options).expect("Failed to create window.");

        let framebuffer = Framebuffer::new(width, height);

        Window {
            window,
            framebuffer,
        }
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open()
    }

    pub fn display(&mut self) {
        self.window
            .update_with_buffer(
                &self.framebuffer.as_buffer(),
                self.framebuffer.width(),
                self.framebuffer.height(),
            )
            .expect("Failed to update window buffer.");

        let (width, height) = self.window.get_size();
        if width != self.framebuffer.width() || height != self.framebuffer.height() {
            self.framebuffer = Framebuffer::new(width, height);
        }
    }

    pub fn framebuffer(&mut self) -> &mut Framebuffer<Vec3> {
        &mut self.framebuffer
    }
}

pub struct Framebuffer<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T: Default + Clone> Framebuffer<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            data: vec![T::default(); width * height],
            width,
            height,
        }
    }
}

impl<T: Copy> Framebuffer<T> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: T) {
        self.data[x + y * self.width] = value
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> T {
        self.data[y * self.width + x]
    }

    pub fn clear(&mut self, value: T) {
        for i in 0..self.data.len() {
            self.data[i] = value;
        }
    }
}

impl Framebuffer<Vec3> {
    pub fn as_buffer(&self) -> Vec<u32> {
        self.data
            .iter()
            .map(|i| {
                let mut array = [0.0; 4];
                i.write_to_slice(&mut array[1..]);
                let bytes = array.map(|float| (float * u8::MAX as f32) as u8);
                u32::from_be_bytes(bytes)
            })
            .collect()
    }
}
