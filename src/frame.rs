use core::panic;

pub type GrayPixel = f32;
pub type RgbPixel = (f32, f32, f32);

pub struct Frame<T> {
    pub width: isize,
    pub height: isize,
    pub data: Vec<T>,
}

impl<T: Default + Clone> Frame<T> {
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
            data: vec![T::default(); (width * height) as usize],
        }
    }

    pub fn get(&self, x: isize, y: isize) -> Option<&T> {
        let i = coordinate_to_index(x, y, self.width);
        if 0 <= i && i < (self.data.len() as isize) {
            Some(&self.data[i as usize])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut T> {
        let i = coordinate_to_index(x, y, self.width);
        if 0 <= i && i < (self.data.len() as isize) {
            Some(&mut self.data[i as usize])
        } else {
            None
        }
    }
}

impl Frame<RgbPixel> {
    pub fn from_rgb24_bytes(width: isize, height: isize, bytes: &[u8]) -> Self {
        if bytes.len() as isize != width * height * 3 {
            panic!(
                "expecting frame bytes {} got {}",
                width * height * 3,
                bytes.len()
            );
        }

        let mut frame = Frame::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let i = (coordinate_to_index(x, y, width) * 3) as usize;
                let red = bytes[i];
                let green = bytes[i + 1];
                let blue = bytes[i + 2];

                *frame.get_mut(x as isize, y as isize).unwrap() =
                    (red.into(), green.into(), blue.into());
            }
        }

        frame
    }

    pub fn to_rgb24_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; (self.width * self.height * 3) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (coordinate_to_index(x, y, self.width) * 3) as usize;
                let (red, green, blue) = clamp8_tuple(*self.get(x, y).unwrap());

                bytes[i] = red;
                bytes[i + 1] = green;
                bytes[i + 2] = blue;
            }
        }

        bytes
    }
}

impl Frame<GrayPixel> {
    pub fn to_rgb24_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; (self.width * self.height * 3) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (coordinate_to_index(x, y, self.width) * 3) as usize;
                let gray = clamp8(*self.get(x, y).unwrap());

                bytes[i] = gray;
                bytes[i + 1] = gray;
                bytes[i + 2] = gray;
            }
        }

        bytes
    }
}

fn coordinate_to_index(x: isize, y: isize, width: isize) -> isize {
    y * width + x
}

pub fn clamp8(value: f32) -> u8 {
    value.clamp(0.0, 255.0) as u8
}

pub fn clamp8_tuple(value: (f32, f32, f32)) -> (u8, u8, u8) {
    (clamp8(value.0), clamp8(value.1), clamp8(value.2))
}
