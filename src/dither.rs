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

pub fn dither_frame_floyd_steinberd_color(frame: &mut Frame) {
    let pixel_offsets = [(1, 0), (-1, 1), (0, 1), (1, 1)];
    let error_coef = [7.0 / 16.0, 3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0];

    for y in 0..frame.height {
        for x in 0..frame.width {
            let (r, g, b) = frame.get_rgb(x, y).unwrap();
            let quantized_r = quantize_8(r);
            let quantized_g = quantize_8(g);
            let quantized_b = quantize_8(b);

            frame.set_rgb(x, y, (quantized_r, quantized_g, quantized_b));

            let err_r = r - quantized_r;
            let err_g = g - quantized_g;
            let err_b = b - quantized_b;

            for ((ox, oy), coef) in pixel_offsets.iter().zip(error_coef) {
                let nx = x + ox;
                let ny = y + oy;
                if let Some((r, g, b)) = frame.get_rgb(nx, ny) {
                    frame.set_rgb(
                        nx,
                        ny,
                        (r + err_r * coef, g + err_g * coef, b + err_b * coef),
                    );
                }
            }
        }
    }
}

fn quantize_8(color: f32) -> f32 {
    let gap = 255.0 / 7.0;

    (color.clamp(0.0, 255.0) / gap).round() * gap
}
