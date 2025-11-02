use std::io::{Read, Write};

use crate::ffmpeg;
use crate::frame::{Frame, Resolution};

#[derive(Debug)]
pub struct DitherOpts {
    pub dither_res: Resolution,
    pub output_res: Resolution,
    pub input_path: String,
    pub output_path: String,
    pub algo: DitherAlgoOpts,
}

#[derive(Debug, Clone)]
pub enum DitherAlgoOpts {
    Atkinson { palette_count: u32 },
    FsColor { palette_count: u32 },
}

pub fn dither_video(opts: DitherOpts) -> Result<(), String> {
    if !opts.dither_res.is_resolved() {
        return Err("dither_res not resolved".to_string());
    }
    if !opts.output_res.is_resolved() {
        return Err("output_res is not resolved".to_string());
    }
    // TODO: Validate Atkinson and FsColor palette_count.

    let (_, _, input_frame_rate) = ffmpeg::get_video_info(&opts.input_path)?;
    let dither_res_w = opts.dither_res.width().unwrap();
    let dither_res_h = opts.dither_res.height().unwrap();
    let output_res_w = opts.output_res.width().unwrap();
    let output_res_h = opts.output_res.height().unwrap();

    // *3 for RGB24
    let mut frame_buf =
        vec![0u8; dither_res_w as usize * dither_res_h as usize * 3];
    let mut frame_reader = ffmpeg::spawn_frame_reader(
        &opts.input_path,
        dither_res_w,
        dither_res_h,
    )?;
    let mut frame_writer_child = ffmpeg::spawn_frame_writer_child(
        dither_res_w,
        dither_res_h,
        output_res_w,
        output_res_h,
        input_frame_rate,
        &opts.output_path,
    )?;
    let mut frame_writer = frame_writer_child
        .stdin
        .take()
        .expect("Expected stdin to be present");

    loop {
        if let Ok(_) = frame_reader.read_exact(&mut frame_buf) {
            match opts.algo {
                DitherAlgoOpts::Atkinson { palette_count } => {
                    dither_frame_atkinson(
                        dither_res_w,
                        dither_res_h,
                        &mut frame_buf,
                        palette_count,
                    );
                }
                DitherAlgoOpts::FsColor { palette_count } => {
                    dither_frame_floyd_steinberg_color(
                        dither_res_w,
                        dither_res_h,
                        &mut frame_buf,
                        palette_count,
                    );
                }
            }

            frame_writer
                .write_all(&frame_buf)
                .map_err(|e| format!("Writing frame buffer failed: {}", e))?;
        } else {
            // EOF, signal to ffmpeg frame writing is done so it can properly finalize
            drop(frame_writer);
            frame_writer_child.wait().unwrap();
            break;
        }
    }

    Ok(())
}

pub fn dither_frame_atkinson(
    width: isize,
    height: isize,
    buffer: &mut [u8],
    palette_count: u32,
) {
    let mut frame = Frame::new(width, height, buffer);

    const PIXEL_OFFSETS: [(isize, isize); 6] =
        [(1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)];
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

pub fn dither_frame_floyd_steinberg_color(
    width: isize,
    height: isize,
    buffer: &mut [u8],
    palette_count: u32,
) {
    let mut frame = Frame::new(width, height, buffer);

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

fn quantize_gap(palette_count: u32) -> f32 {
    255.0 / (palette_count as f32 - 1.0)
}

fn quantize(color: f32, gap: f32) -> f32 {
    (color / gap + 0.5).floor() * gap
}
