use std::io;
use std::process::{Child, ChildStdout, Command, Stdio};

pub fn spawn_frame_reader(path: &str) -> io::Result<ChildStdout> {
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(child.stdout.take().unwrap())
}

pub fn spawn_child_frame_writer(
    width: i32,
    height: i32,
    frame_rate: f32,
    out_path: &str,
) -> io::Result<Child> {
    let child = Command::new("ffmpeg")
        .args(&[
            "-v",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            &format!("{}x{}", width, height),
            "-r",
            &frame_rate.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            out_path,
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    Ok(child)
}

/// Get width, height and frame rate of a video.
pub fn get_video_info(path: &str) -> Result<(i32, i32, f32), String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,avg_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed to start: {}", e))?;

    if !output.status.success() {
        let reason = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or("SIGNAL".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!("ffprobe exited with {}\n{}", reason, stderr));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| format!("ffprobe yielded an invalid UTF-8 output: {}", e))?;
    let lines: Vec<&str> = stdout.lines().collect();
    let width: i32 = lines[0].parse().unwrap();
    let height: i32 = lines[1].parse().unwrap();
    let frame_rate = {
        let parts: Vec<&str> = lines[2].split("/").collect();
        let (num, denom): (f32, f32) = (parts[0].parse().unwrap(), parts[1].parse().unwrap());
        num / denom
    };

    Ok((width, height, frame_rate))
}
