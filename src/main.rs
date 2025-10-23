use clap::Parser;
use libc::{SIGINT, SIGTERM, c_int, signal};
use std::fs;

use crate::dither::{dither_frame_atkinson, dither_frame_floyd_steinberg_color};

mod args;
mod dither;
mod ffmpeg;
mod frame;

extern "C" fn handle_signal(_sig: c_int) {}

fn main() {
    // Ignore these signals from the OS, instead handle ffmpeg shutdown as
    // normal error so we have a chance to clean up.
    unsafe {
        signal(SIGINT, handle_signal as usize);
        signal(SIGTERM, handle_signal as usize);
    }

    let args = args::Args::parse();
    let temp_output = &format!("dither_some_{}", args.output);
    let actual_res = args.actual_res.unwrap();

    if let Err(e) = ffmpeg::dither_frames_with(
        &args.input,
        &temp_output,
        actual_res.width(),
        actual_res.height(),
        |width, height, buffer| {
            let (width, height) = (width as isize, height as isize);

            match args.algorithm {
                args::Algorithm::Atkinson => {
                    dither_frame_atkinson(width, height, buffer, args.palette_count)
                }
                args::Algorithm::FsColor => {
                    dither_frame_floyd_steinberg_color(width, height, buffer, args.palette_count)
                }
            }
        },
    ) {
        let _ = fs::remove_file(temp_output);
        eprint!("{}", e);
        return;
    }

    let result =
        ffmpeg::copy_streams_or_aac_transcode_audio(&temp_output, &args.input, &args.output);
    let _ = fs::remove_file(temp_output);
    result.unwrap();
}
