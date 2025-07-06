pub type GrayPixel = f32;
pub type RgbPixel = (f32, f32, f32);

pub struct Frame {
    pub width: isize,
    pub height: isize,
    pub data: Vec<f32>,
}

impl Frame {
    pub fn new(width: isize, height: isize) -> Self {
        if width < 0 {
            panic!("negative width");
        }
        if height < 0 {
            panic!("negative height");
        }

        Frame {
            width: width,
            height: height,
            data: vec![0.0; (width * height * 3) as usize],
        }
    }

    pub fn from_rgb24_bytes(width: isize, height: isize, buffer: &[u8]) -> Self {
        let expected_size = (width * height * 3) as usize;
        if buffer.len() != expected_size {
            panic!(
                "expecting slice size {} got {}",
                expected_size,
                buffer.len()
            );
        }

        Frame {
            width: width,
            height: height,
            data: buffer.iter().map(|n| *n as f32).collect(),
        }
    }

    pub fn get_rgb(&self, x: isize, y: isize) -> Option<RgbPixel> {
        let ix = self.coordinate_to_index(x, y) * 3;
        if 0 <= ix && ix < self.data.len() as isize {
            Some((
                self.data[ix as usize],
                self.data[(ix + 1) as usize],
                self.data[(ix + 2) as usize],
            ))
        } else {
            None
        }
    }

    pub fn get_gray(&self, x: isize, y: isize) -> Option<GrayPixel> {
        if let Some((r, g, b)) = self.get_rgb(x, y) {
            Some(0.299 * r + 0.587 * g + 0.114 * b)
        } else {
            None
        }
    }

    pub fn set_rgb(&mut self, x: isize, y: isize, new_rgb: RgbPixel) -> bool {
        let ix = self.coordinate_to_index(x, y) * 3;
        if 0 <= ix && ix < self.data.len() as isize {
            let (r, g, b) = new_rgb;
            self.data[ix as usize] = r;
            self.data[(ix + 1) as usize] = g;
            self.data[(ix + 2) as usize] = b;

            true
        } else {
            false
        }
    }

    pub fn set_gray(&mut self, x: isize, y: isize, new_gray: GrayPixel) -> bool {
        self.set_rgb(x, y, (new_gray, new_gray, new_gray))
    }

    pub fn to_rgb24_bytes(&self, buffer: &mut [u8]) {
        if buffer.len() != self.data.len() {
            panic!(
                "expecting slice size {} got {}",
                self.data.len(),
                buffer.len()
            );
        }

        for (i, v) in self.data.iter().enumerate() {
            buffer[i] = v.clamp(0.0, 255.0) as u8;
        }
    }

    fn coordinate_to_index(&self, x: isize, y: isize) -> isize {
        y * self.width + x
    }
}
