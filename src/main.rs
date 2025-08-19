use clap::{Parser, ValueEnum};
use libc::{SIGINT, SIGTERM, c_int, signal};
use std::fs;

use crate::dither::{dither_frame_atkinson, dither_frame_floyd_steinberg_color};

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

extern "C" fn handle_signal(_sig: c_int) {}

fn main() {
    // Ignore these signals from the OS, instead handle ffmpeg shutdown as
    // normal error so we have a chance to clean up.
    unsafe {
        signal(SIGINT, handle_signal as usize);
        signal(SIGTERM, handle_signal as usize);
    }

    let args = Args::parse();
    let temp_output = &format!("dither_some_{}", args.output);

    if let Err(e) =
        ffmpeg::dither_frames_with(&args.input, &temp_output, |width, height, buffer| {
            let (width, height) = (width as isize, height as isize);

            match args.algorithm {
                Algorithm::Atkinson => {
                    dither_frame_atkinson(width, height, buffer, args.palette_count)
                }
                Algorithm::FsColor => {
                    dither_frame_floyd_steinberg_color(width, height, buffer, args.palette_count)
                }
            }
        })
    {
        let _ = fs::remove_file(temp_output);
        eprint!("{}", e);
        return;
    }

    let result =
        ffmpeg::copy_streams_or_aac_transcode_audio(&temp_output, &args.input, &args.output);
    let _ = fs::remove_file(temp_output);
    result.unwrap();
}
