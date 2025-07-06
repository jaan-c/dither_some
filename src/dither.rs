use crate::frame::Frame;

pub fn dither_frame_atkinson(frame: &mut Frame) {
    let pixel_offsets = [(1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)];

    for y in 0..frame.height {
        for x in 0..frame.width {
            let pixel = frame.get_gray(x, y).unwrap();
            let quantized = { if pixel < 128.0 { 0.0 } else { 255.0 } };
            let error = pixel - quantized;
            let eight_error = error * (1.0 / 8.0);

            frame.set_gray(x, y, quantized);

            for (ox, oy) in pixel_offsets {
                let nx = x + ox;
                let ny = y + oy;
                if let Some(p) = frame.get_gray(nx, ny) {
                    frame.set_gray(nx, ny, p + eight_error);
                }
            }
        }
    }
}
