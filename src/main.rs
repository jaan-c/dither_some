use std::env;
use std::io::{Read, Write};

use crate::dither::dither_frame_atkinson;
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
    let mut frame_buf = vec![0u8; frame_size];

    let mut frame_reader = ffmpeg::spawn_frame_reader(in_path).unwrap();
    let mut child = ffmpeg::spawn_child_frame_writer(width, height, frame_rate, out_path).unwrap();
    let mut frame_writer = child.stdin.take().unwrap();

    loop {
        if let Ok(_) = frame_reader.read_exact(&mut frame_buf) {
            let mut frame = Frame::from_rgb24_bytes(width as isize, height as isize, &frame_buf);

            dither_frame_atkinson(&mut frame);
            frame.to_rgb24_bytes(&mut frame_buf);

            frame_writer.write_all(&frame_buf).unwrap();
        } else {
            break;
        }
    }
}
