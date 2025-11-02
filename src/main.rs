use clap::Parser;
use libc::{SIGINT, SIGTERM, c_int, signal};
use std::fs;
use std::path;

mod cli;
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

    let args = cli::CliArgs::parse();

    if path::Path::new(&args.output).exists() {
        println!("Output '{}' already exists.", args.output);
        return;
    }

    let (input_w, input_h, _) = ffmpeg::get_video_info(&args.input).unwrap();

    let temp_output_path = format!("dither_some_{}", args.output);
    let input_res = frame::Resolution::new(input_w as isize, input_h as isize);

    let dither_res = match args.dither_res {
        Some(dither_res) => dither_res.resolve_fields(&input_res).unwrap(),
        None => input_res.clone(),
    };
    let output_res = match args.output_res {
        Some(output_res) => output_res.resolve_fields(&input_res).unwrap(),
        None => input_res.clone(),
    };
    let dither_algo_opts = match args.algorithm {
        cli::CliAlgorithm::Atkinson { palette_count } => {
            dither::DitherAlgoOpts::Atkinson {
                palette_count: palette_count,
            }
        }
        cli::CliAlgorithm::FsColor { palette_count } => {
            dither::DitherAlgoOpts::FsColor {
                palette_count: palette_count,
            }
        }
    };
    let dither_opts = dither::DitherOpts {
        dither_res: dither_res,
        output_res: output_res,
        input_path: args.input.clone(),
        output_path: temp_output_path.clone(),
        algo: dither_algo_opts,
    };

    if let Err(e) = dither::dither_video(dither_opts) {
        let _ = fs::remove_file(temp_output_path);
        eprint!("{}", e);
        return;
    }

    let result = ffmpeg::copy_streams_or_aac_transcode_audio(
        &temp_output_path,
        &args.input,
        &args.output,
    );
    let _ = fs::remove_file(temp_output_path);
    result.unwrap();
}
