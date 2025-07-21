pub type GrayPixel = f32;
pub type RgbPixel = (f32, f32, f32);

/// A wrapper around \[u8] to manipulate it like an f32 RGB24 matrix.
pub struct Frame<'a> {
    pub width: isize,
    pub height: isize,
    data: &'a mut [u8],
}

impl<'a> Frame<'a> {
    pub fn new(width: isize, height: isize, data: &'a mut [u8]) -> Self {
        assert!(width > -1);
        assert!(height > -1);
        assert!(data.len() == (width * height * 3) as usize);

        return Frame {
            width: width,
            height: height,
            data: data,
        };
    }

    pub fn get_rgb(&self, x: isize, y: isize) -> Option<RgbPixel> {
        if let Some(i) = self.coordinate_to_index(x, y) {
            // This is safe since we already checked index validity; each pixel
            // occupies 3 slots in self.data, if i is safe, i+1 and i+2 is safe.
            unsafe {
                Some((
                    *self.data.get_unchecked(i) as f32,
                    *self.data.get_unchecked(i + 1) as f32,
                    *self.data.get_unchecked(i + 2) as f32,
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
        if let Some(i) = self.coordinate_to_index(x, y) {
            let (r, g, b) = new_rgb;

            // This is safe since we already checked index validity; each pixel
            // occupies 3 slots in self.data, if i is safe, i+1 and i+2 is safe.
            unsafe {
                *self.data.get_unchecked_mut(i) = r as u8;
                *self.data.get_unchecked_mut(i + 1) = g as u8;
                *self.data.get_unchecked_mut(i + 2) = b as u8;
            }

            true
        } else {
            false
        }
    }

    pub fn set_gray(&mut self, x: isize, y: isize, new_gray: GrayPixel) -> bool {
        self.set_rgb(x, y, (new_gray, new_gray, new_gray))
    }

    fn coordinate_to_index(&self, x: isize, y: isize) -> Option<usize> {
        if 0 <= x && x <= self.width && 0 <= y && y < self.height {
            Some(((y * self.width + x) * 3) as usize)
        } else {
            None
        }
    }
}
