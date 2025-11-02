use std::str::FromStr;

use clap::{Parser, Subcommand};

use crate::frame::Resolution;

#[derive(Parser, Debug)]
#[command(version, subcommand_value_name = "ALGORITHM")]
pub struct CliArgs {
    /// The actual resolution of the image when it is dithered. Defaults to input resolution.
    #[arg(long, allow_hyphen_values = true)]
    pub dither_res: Option<Resolution>,

    /// The output resolution. Defaults to input resolution.
    #[arg(long, allow_hyphen_values = true)]
    pub output_res: Option<Resolution>,

    /// Path of video to dither.
    #[arg(index = 1)]
    pub input: String,

    /// Path where to save dithered video.
    #[arg(index = 2)]
    pub output: String,

    #[command(subcommand)]
    pub algorithm: CliAlgorithm,
}

#[derive(Debug, Subcommand)]
pub enum CliAlgorithm {
    /// Apply Atkinson dithering algorithm.
    Atkinson {
        #[arg(short, long, value_parser = clap::value_parser!(u32).range(2..=256))]
        palette_count: u32,
    },

    /// Apply colored Floyd-Steinberg dithering algorithm.
    FsColor {
        #[arg(short, long, value_parser = clap::value_parser!(u32).range(2..=256))]
        palette_count: u32,
    },
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
