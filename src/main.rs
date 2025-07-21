use clap::{Parser, ValueEnum};
use std::fs;

use crate::dither::{dither_frame_atkinson, dither_frame_floyd_steinberg_color};
use crate::frame::Frame;

mod dither;
mod ffmpeg;
mod frame;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Algorithm to be used for dithering.
    #[arg(short, long)]
    algorithm: Algorithm,

    /// Restricts palette by specified count.
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(2..=255))]
    palette_count: u8,

    /// Path of video to dither.
    #[arg(index = 1)]
    input: String,

    /// Path where to store dithered video.
    #[arg(index = 2)]
    output: String,
}

#[derive(Debug, Clone, ValueEnum)]
enum Algorithm {
    Atkinson,
    FsColor,
}

fn main() {
    let args = Args::parse();
    let temp_output = &format!("dither_some_{}", args.output);

    ffmpeg::dither_frames_with(&args.input, &temp_output, |width, height, frame_buf| {
        let mut frame = Frame::new(width as isize, height as isize, frame_buf);

        match args.algorithm {
            Algorithm::Atkinson => dither_frame_atkinson(&mut frame, args.palette_count),
            Algorithm::FsColor => {
                dither_frame_floyd_steinberg_color(&mut frame, args.palette_count)
            }
        }
    })
    .unwrap();

    let result =
        ffmpeg::copy_streams_or_aac_transcode_audio(&temp_output, &args.input, &args.output);
    fs::remove_file(temp_output).ok();
    result.unwrap();
}
