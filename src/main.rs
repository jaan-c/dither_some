use std::env;
use std::io::{self, Read, Write};

use crate::dither::{dither_frame_atkinson, rgb_to_gray_frame};
use crate::frame::Frame;

mod dither;
mod ffmpeg;
mod frame;

fn main() {
    let args: Vec<String> = env::args().collect();
    let in_path = &args[1];
    let out_path = &args[2];
    let (width, height, frame_rate) = ffmpeg::get_video_info(in_path).unwrap();
    let frame_size = (width * height * 3) as usize;
    let mut buf = vec![0u8; frame_size];
    let mut frame_reader = ffmpeg::spawn_frame_reader(in_path).unwrap();
    let mut frame_child_writer =
        ffmpeg::spawn_child_writer(width, height, frame_rate, out_path).unwrap();
    let mut frame_writer = frame_child_writer.stdin.take().unwrap();

    loop {
        if let Ok(_) = frame_reader.read_exact(&mut buf) {
            let rgb_frame = Frame::from_rgb24_bytes(width as isize, height as isize, &buf);
            // NOTE: We're making 2 gigantic Vecs starting here.
            let mut gray_frame = rgb_to_gray_frame(&rgb_frame);
            dither_frame_atkinson(&mut gray_frame);

            let bytes = gray_frame.to_rgb24_bytes();
            frame_writer.write_all(&bytes).unwrap();
        } else {
            break;
        }
    }
}
