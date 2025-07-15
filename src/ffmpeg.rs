use std::io;
use std::process::{Child, ChildStdout, Command, Stdio};

pub fn spawn_frame_reader(path: &str) -> io::Result<ChildStdout> {
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(child.stdout.take().expect("Expected stdout is present"))
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
            "-n",
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

/// Copies src_video and src_audio's corresponding streams to dest, but if
/// FFmpeg fails, it tries to transcode the audio to AAC.
///
/// AAC because it's widely used audio encoding for MP4, and I'm assuming that's
/// the only container we're working with now.
pub fn copy_streams_or_aac_transcode_audio(
    src_video: &str,
    src_audio: &str,
    dest: &str,
) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args(&[
            "-v", "error", "-i", src_video, "-i", src_audio, "-c:v", "copy", "-c:a", "copy",
            "-map", "0:v:0", "-map", "1:a:0", "-n", dest,
        ])
        .status()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    if !status.success() {
        let output = Command::new("ffmpeg")
            .args(&[
                "-v", "error", "-i", src_video, "-i", src_audio, "-c:v", "copy", "-c:a", "copy",
                "-map", "0:v:0", "-map", "1:a:0", "-n", dest,
            ])
            .output()
            .map_err(|e| format!("Failed to start fallback ffmpeg: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Fallback ffmpeg failed: {}", stderr));
        }
    }

    Ok(())
}
