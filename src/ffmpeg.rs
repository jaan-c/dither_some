use std::io::{Read, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

pub fn dither_frames_with<F>(input: &str, output: &str, dither_fn: F) -> Result<(), String>
where
    F: Fn(usize, usize, &mut [u8]),
{
    let (width, height, frame_rate) = get_video_info(input)?;

    let mut frame_buf = vec![0u8; width * height * 3]; // *3 for RGB24
    let mut frame_reader = spawn_frame_reader(input)?;
    let mut frame_writer_child = spawn_frame_writer_child(width, height, frame_rate, output)?;
    let mut frame_writer = frame_writer_child
        .stdin
        .take()
        .expect("Expected stdin to be present");

    loop {
        if let Ok(_) = frame_reader.read_exact(&mut frame_buf) {
            dither_fn(width, height, &mut frame_buf);
            frame_writer
                .write_all(&frame_buf)
                .map_err(|e| format!("Writing frame buffer failed: {}", e))?;
        } else {
            // EOF, signal to ffmpeg we're done so it can properly finalize the output.
            drop(frame_writer);
            frame_writer_child.wait().unwrap();
            break;
        }
    }

    Ok(())
}

fn spawn_frame_reader(path: &str) -> Result<ChildStdout, String> {
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffmpeg frame reader failed to start: {}", e))?;

    Ok(child.stdout.take().expect("Expected stdout to be present"))
}

fn spawn_frame_writer_child(
    width: usize,
    height: usize,
    frame_rate: f32,
    path: &str,
) -> Result<Child, String> {
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
            path,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffmpeg frame writer failed to start: {}", e))?;

    Ok(child)
}

/// Get width, height and frame rate of a video.
fn get_video_info(path: &str) -> Result<(usize, usize, f32), String> {
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

        return Err(format!("ffprobe exited with {}: {}", reason, stderr));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| format!("ffprobe yielded an invalid UTF-8 output: {}", e))?;
    let lines: Vec<&str> = stdout.lines().collect();
    let width: usize = lines[0].parse().map_err(|_| "Parsing width failed")?;
    let height: usize = lines[1].parse().map_err(|_| "Parsing height failed")?;
    let frame_rate = {
        let parts: Vec<&str> = lines[2].split("/").collect();
        let (num, denom): (f32, f32) = (
            parts[0].parse().map_err(|_| "Parsing frame_rate failed")?,
            parts[1].parse().map_err(|_| "Parsing frame_rate failed")?,
        );
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
        .map_err(|e| format!("ffmpeg failed to start: {}", e))?;

    if !status.success() {
        let output = Command::new("ffmpeg")
            .args(&[
                "-v", "error", "-i", src_video, "-i", src_audio, "-c:v", "copy", "-c:a", "copy",
                "-map", "0:v:0", "-map", "1:a:0", "-n", dest,
            ])
            .output()
            .map_err(|e| format!("ffmpeg fallback failed to start: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg fallback failed: {}", stderr));
        }
    }

    Ok(())
}
