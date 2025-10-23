use std::str::FromStr;

use clap::{Parser, ValueEnum};

use crate::frame::Resolution;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Algorithm to be used for dithering.
    #[arg(short, long)]
    pub algorithm: Algorithm,

    /// Restricts palette by specified count.
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(2..=255))]
    pub palette_count: u8,

    /// The actual resolution of the image that will be dithered.
    #[arg(long)]
    pub actual_res: Option<Resolution>,

    /// The resolution of the frames in the output video.
    #[arg(long)]
    pub output_res: Option<Resolution>,

    /// Path of video to dither.
    #[arg(index = 1)]
    pub input: String,

    /// Path where to store dithered video.
    #[arg(index = 2)]
    pub output: String,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Algorithm {
    Atkinson,
    FsColor,
}

impl FromStr for Resolution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(format!("Expected WIDTHxHEIGHT, got '{}'", s));
        }

        let width = parts[0]
            .parse::<isize>()
            .map_err(|_| "Invalid width".to_string())?;
        let height = parts[1]
            .parse::<isize>()
            .map_err(|_| "Invalid width".to_string())?;

        Ok(Resolution::new(width, height))
    }
}
