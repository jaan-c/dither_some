use clap::{Parser, ValueEnum};
use std::fs;
use std::io::{Read, Write};

use crate::dither::{dither_frame_atkinson, dither_frame_floyd_steinberd_color};
use crate::ffmpeg::copy_streams_or_aac_transcode_audio;
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
    #[arg(short, long, value_parser = clap::value_parser!(i32).range(2..=255))]
    palette_count: i32,

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

    let (width, height, frame_rate) = ffmpeg::get_video_info(&args.input).unwrap();

    let frame_size = (width * height * 3) as usize;
    let mut frame_buf = vec![0u8; frame_size];

    let mut frame_reader = ffmpeg::spawn_frame_reader(&args.input).unwrap();
    let mut frame_writer_child =
        ffmpeg::spawn_child_frame_writer(width, height, frame_rate, temp_output).unwrap();
    let mut frame_writer = frame_writer_child.stdin.take().unwrap();

    loop {
        if let Ok(_) = frame_reader.read_exact(&mut frame_buf) {
            let mut frame = Frame::from_rgb24_bytes(width as isize, height as isize, &frame_buf);

            match args.algorithm {
                Algorithm::Atkinson => dither_frame_atkinson(&mut frame, args.palette_count),
                Algorithm::FsColor => {
                    dither_frame_floyd_steinberd_color(&mut frame, args.palette_count)
                }
            }

            frame.to_rgb24_bytes(&mut frame_buf);
            frame_writer.write_all(&frame_buf).unwrap();
        } else {
            // Signal to ffmpeg we're done so it can properly finalize.
            drop(frame_writer);
            frame_writer_child.wait().unwrap();
            break;
        }
    }

    let result = copy_streams_or_aac_transcode_audio(&temp_output, &args.input, &args.output);
    fs::remove_file(temp_output).ok();
    result.unwrap();
}
