use std::{path::Path, process::Command};

pub fn merge<P: AsRef<Path>>(video: &P, audio: &P, output: &P) -> Result<bool, ()> {
    // Check if output is existed, if yes, remove it
    if output.as_ref().exists() {
        std::fs::remove_file(output.as_ref()).unwrap();
    }

    let args = [
        "-add",
        video.as_ref().to_str().unwrap(),
        "-add",
        audio.as_ref().to_str().unwrap(),
        output.as_ref().to_str().unwrap(),
    ];

    let output = Command::new("mp4box")
        .args(&args)
        .output()
        .expect("failed to execute process");

    let success = output.status.success();
    Ok(success)
}
