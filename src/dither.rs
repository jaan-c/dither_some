use crate::frame::{Frame, GrayPixel, RgbPixel};

pub fn rgb_to_gray_frame(frame: &Frame<RgbPixel>) -> Frame<GrayPixel> {
    let mut gray_frame = Frame::new(frame.width, frame.height);
    for y in 0..(frame.height as isize) {
        for x in 0..(frame.width as isize) {
            let (r, g, b) = frame.get(x, y).unwrap();
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
            *gray_frame.get_mut(x, y).unwrap() = gray;
        }
    }

    gray_frame
}

pub fn dither_frame_atkinson(frame: &mut Frame<GrayPixel>) {
    let pixel_offsets = [(1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)];

    for y in 0..frame.height {
        for x in 0..frame.width {
            let pixel = *frame.get(x, y).unwrap();
            let quantized = { if pixel < 128.0 { 0.0 } else { 255.0 } };
            let error = pixel - quantized;
            let eight_error = error * (1.0 / 8.0);

            *frame.get_mut(x, y).unwrap() = quantized;

            for (dx, dy) in pixel_offsets {
                if let Some(p) = frame.get_mut(x + dx, y + dy) {
                    *p += eight_error;
                }
            }
        }
    }
}
