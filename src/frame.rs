#[derive(Debug, Clone)]
pub struct Resolution {
    _width: isize,
    _height: isize,
}

impl Resolution {
    pub fn new(width: isize, height: isize) -> Resolution {
        Resolution {
            _width: width,
            _height: height,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self._width >= 0 && self._height >= 0
    }

    pub fn width(&self) -> Option<usize> {
        if self._width >= 0 {
            Some(self._width as usize)
        } else {
            None
        }
    }

    pub fn height(&self) -> Option<usize> {
        if self._height >= 0 {
            Some(self._height as usize)
        } else {
            None
        }
    }

    pub fn resolve_fields(&self, relative_to: Resolution) -> Result<Resolution, String> {
        if !relative_to.is_resolved() {
            return Err("relative_to has to be resolved".to_string());
        }

        let ratio = relative_to._width as f32 / relative_to._height as f32;

        if self._width >= 0 && self._height >= 0 {
            Ok(self.clone())
        } else if self._width < 0 && self._height < 0 {
            Err("Either field has to at least be positive.".to_string())
        } else if self._width < 0 {
            let resolved_width = self._height as f32 * ratio;
            let resolved_width = match self._width {
                -2 => round_even(resolved_width),
                _ => resolved_width.round(),
            };

            Ok(Resolution {
                _width: resolved_width as isize,
                _height: self._height,
            })
        } else {
            let resolved_height = self._width as f32 / ratio;
            let resolved_height = match self._height {
                -2 => round_even(resolved_height),
                _ => resolved_height.round(),
            };

            Ok(Resolution {
                _width: self._width,
                _height: resolved_height as isize,
            })
        }
    }
}

fn round_even(n: f32) -> f32 {
    (n / 2.0).round() * 2.0
}

pub type GrayPixel = f32;
pub type RgbPixel = (f32, f32, f32);

/// A wrapper around \[u8] to manipulate it like an f32 RGB24 matrix. f32 values
/// that don't fit in u8 when set_*, will be clamped.
pub struct Frame<'a> {
    pub width: isize,
    pub height: isize,
    buffer: &'a mut [u8],
}

impl<'a> Frame<'a> {
    pub fn new(width: isize, height: isize, buffer: &'a mut [u8]) -> Self {
        assert!(width > -1);
        assert!(height > -1);
        assert!(buffer.len() == (width * height * 3) as usize);

        return Frame {
            width: width,
            height: height,
            buffer,
        };
    }

    pub fn get_rgb(&self, x: isize, y: isize) -> Option<RgbPixel> {
        if let Some(i) = self.coordinate_to_index(x, y) {
            // This is safe since we already checked index validity; each pixel
            // occupies 3 slots in self.data, if i is safe, i+1 and i+2 is safe.
            unsafe {
                Some((
                    *self.buffer.get_unchecked(i) as f32,
                    *self.buffer.get_unchecked(i + 1) as f32,
                    *self.buffer.get_unchecked(i + 2) as f32,
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
                *self.buffer.get_unchecked_mut(i) = r as u8;
                *self.buffer.get_unchecked_mut(i + 1) = g as u8;
                *self.buffer.get_unchecked_mut(i + 2) = b as u8;
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
        if 0 <= x && x < self.width && 0 <= y && y < self.height {
            Some(((y * self.width + x) * 3) as usize)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn new_panics_with_negative_width() {
        Frame::new(-1, 1, &mut vec![0u8; 3]);
    }

    #[test]
    #[should_panic]
    fn new_panics_with_negative_height() {
        Frame::new(1, -1, &mut vec![0u8; 3]);
    }

    #[test]
    #[should_panic]
    fn new_panics_with_mismatched_buffer_size() {
        Frame::new(1, 2, &mut vec![0u8; 12]);
    }

    #[test]
    fn get_rgb_returns_correct_pixel() {
        let mut buf = vec![0u8; 3];
        let frame = Frame::new(1, 1, &mut buf);

        assert_eq!(frame.get_rgb(0, 0).unwrap(), (0.0, 0.0, 0.0));
    }

    #[test]
    fn get_rgb_returns_none_on_out_of_bounds() {
        let mut buf = vec![0u8; 3];
        let frame = Frame::new(1, 1, &mut buf);

        assert_eq!(frame.get_rgb(3, 3), None);
    }

    #[test]
    fn get_rgb_returns_none_on_negative() {
        let mut buf = vec![0u8; 3];
        let frame = Frame::new(1, 1, &mut buf);

        assert_eq!(frame.get_rgb(-1, -1), None);
    }

    #[test]
    fn get_gray_returns_correct_value() {
        let mut buf = vec![0u8; 3];
        let mut frame = Frame::new(1, 1, &mut buf);

        let (r, g, b) = (100.0, 150.0, 200.0);
        frame.set_rgb(0, 0, (r, g, b));

        let gray = frame.get_gray(0, 0).unwrap();
        let expected = 0.299 * r + 0.587 * g + 0.114 * b;

        assert!((gray - expected).abs() < 1e-6);
    }

    #[test]
    fn get_gray_returns_none_on_out_of_bounds() {
        let mut buf = vec![0u8; 3];
        let frame = Frame::new(1, 1, &mut buf);

        assert_eq!(frame.get_gray(5, 5), None);
    }

    #[test]
    fn set_rgb_writes_correct_pixel() {
        let mut buf = vec![0u8; 3];
        let mut frame = Frame::new(1, 1, &mut buf);

        let success = frame.set_rgb(0, 0, (10.0, 20.0, 30.0));
        assert!(success);
        assert_eq!(frame.get_rgb(0, 0).unwrap(), (10.0, 20.0, 30.0));
    }

    #[test]
    fn set_rgb_returns_false_on_out_of_bounds() {
        let mut buf = vec![0u8; 3];
        let mut frame = Frame::new(1, 1, &mut buf);

        assert!(!frame.set_rgb(2, 2, (1.0, 1.0, 1.0)));
    }

    #[test]
    fn set_gray_writes_correct_pixel() {
        let mut buf = vec![0u8; 3];
        let mut frame = Frame::new(1, 1, &mut buf);

        let success = frame.set_gray(0, 0, 128.0);
        assert!(success);
        assert_eq!(frame.get_rgb(0, 0).unwrap(), (128.0, 128.0, 128.0));
    }

    #[test]
    fn set_gray_returns_false_on_negative_index() {
        let mut buf = vec![0u8; 3];
        let mut frame = Frame::new(1, 1, &mut buf);

        assert!(!frame.set_gray(-1, 0, 100.0));
    }
}
