use crate::frame::Frame;

pub fn dither_frame_atkinson(frame: &mut Frame, palette_count: u8) {
    const PIXEL_OFFSETS: [(isize, isize); 6] = [(1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)];
    let gap = quantize_gap(palette_count);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let pixel = frame.get_gray(x, y).unwrap();
            let quantized = quantize(pixel, gap);
            let error = pixel - quantized;
            let eight_error = error / 8.0;

            frame.set_gray(x, y, quantized);

            for (ox, oy) in PIXEL_OFFSETS {
                let nx = x + ox;
                let ny = y + oy;
                if let Some(p) = frame.get_gray(nx, ny) {
                    frame.set_gray(nx, ny, p + eight_error);
                }
            }
        }
    }
}

pub fn dither_frame_floyd_steinberg_color(frame: &mut Frame, palette_count: u8) {
    const OFFSET_COEF: [((isize, isize), f32); 4] = [
        ((1, 0), 7.0 / 16.0),
        ((-1, 1), 3.0 / 16.0),
        ((0, 1), 5.0 / 16.0),
        ((1, 1), 1.0 / 16.0),
    ];
    let gap = quantize_gap(palette_count);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let (r, g, b) = frame.get_rgb(x, y).unwrap();
            let quantized_r = quantize(r, gap);
            let quantized_g = quantize(g, gap);
            let quantized_b = quantize(b, gap);

            frame.set_rgb(x, y, (quantized_r, quantized_g, quantized_b));

            let err_r = r - quantized_r;
            let err_g = g - quantized_g;
            let err_b = b - quantized_b;

            for ((ox, oy), coef) in OFFSET_COEF {
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

fn quantize_gap(palette_count: u8) -> f32 {
    255.0 / (palette_count as f32 - 1.0)
}

fn quantize(color: f32, gap: f32) -> f32 {
    (color / gap + 0.5).floor() * gap
}
