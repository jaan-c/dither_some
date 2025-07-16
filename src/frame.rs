pub type GrayPixel = f32;
pub type RgbPixel = (f32, f32, f32);

/// Stores frame data in a flat Vec, represented like RGB24, but uses f32
/// instead of 8 bits per channel to allow overflow for error diffusion.
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

        let mut data = Vec::with_capacity(buffer.len());
        data.extend(buffer.iter().map(|&n| n as f32));

        Frame {
            width: width,
            height: height,
            data: data,
        }
    }

    pub fn get_rgb(&self, x: isize, y: isize) -> Option<RgbPixel> {
        let i = self.coordinate_to_index(x, y);
        if self.is_index_valid(i) {
            let i = i as usize;

            // This is safe since we already checked index validity; each pixel
            // occupies 3 slots in self.data, if i is safe, i+1 and i+2 is safe.
            unsafe {
                Some((
                    *self.data.get_unchecked(i),
                    *self.data.get_unchecked(i + 1),
                    *self.data.get_unchecked(i + 2),
                ))
            }
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
        let i = self.coordinate_to_index(x, y);
        if self.is_index_valid(i) {
            let i = i as usize;
            let (r, g, b) = new_rgb;

            // This is safe since we already checked index validity; each pixel
            // occupies 3 slots in self.data, if i is safe, i+1 and i+2 is safe.
            unsafe {
                *self.data.get_unchecked_mut(i) = r;
                *self.data.get_unchecked_mut(i + 1) = g;
                *self.data.get_unchecked_mut(i + 2) = b;
            }

            true
        } else {
            false
        }
    }

    pub fn set_gray(&mut self, x: isize, y: isize, new_gray: GrayPixel) -> bool {
        self.set_rgb(x, y, (new_gray, new_gray, new_gray))
    }

    pub fn to_rgb24_bytes(&self, dest: &mut [u8]) {
        if dest.len() != self.data.len() {
            panic!(
                "expecting slice size {} got {}",
                self.data.len(),
                dest.len()
            );
        }

        for i in 0..self.data.len() {
            unsafe {
                *dest.get_unchecked_mut(i) = *self.data.get_unchecked(i) as u8;
            }
        }
    }

    fn is_index_valid(&self, index: isize) -> bool {
        0 <= index && index < self.data.len() as isize
    }

    fn coordinate_to_index(&self, x: isize, y: isize) -> isize {
        (y * self.width + x) * 3
    }
}
