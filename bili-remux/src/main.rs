use std::{
    fs::{self, create_dir_all, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    path::PathBuf,
};

use info::{Sources, VideoInfo};

mod cmd;
mod info;

fn main() {
    let home_dir = home::home_dir().unwrap();
    let work_dir = home_dir.join("Movies").join("bilibili");

    // Loop all directory in work_dir
    for entry in fs::read_dir(work_dir).unwrap() {
        let entry = entry.unwrap();
        let dir_path = entry.path();
        if dir_path.is_dir() {
            let source = validate_source_files(&dir_path);
            if source.is_err() {
                println!("Not validate source");
                continue;
            }
            let source = source.unwrap();
            let success = cmd::merge(&source.video, &source.audio, &source.output).unwrap();
            if success {
                println!("Successfully merged video and audio");
            } else {
                println!("Failed to merge video and audio");
            }
            fs::remove_dir_all(&dir_path.join("tmp")).unwrap();
            println!("Removed temporary files");
        }
    }
}

fn validate_source_files(dir: &PathBuf) -> Result<Sources, ()> {
    let info = dir.join(".videoInfo");
    let mut video = PathBuf::new();
    let mut audio = PathBuf::new();
    for (_, entry) in dir.read_dir().unwrap().enumerate() {
        let path = entry.unwrap().path();
        if path.is_file() {
            match path.extension() {
                Some(ext) => {
                    if ext == "m4s" {
                        let file_stem = path.file_stem().unwrap().to_str().unwrap();
                        if file_stem.contains("30280") {
                            audio = path;
                        } else {
                            video = path;
                        }
                    }
                }
                None => (),
            }
        }
    }
    if video.is_file() && audio.is_file() {
        // Copy video and audio to new directory
        let video_info = extract_video_info(&info);
        let output_file = format!("[P{}]{}.mp4", video_info.fragment, video_info.tab_name);
        let output_path = dir.parent().unwrap().join(&output_file);

        let new_dir = dir.join("tmp");
        let video_path = new_dir.join("video.m4s");
        let audio_path = new_dir.join("audio.m4s");
        create_dir_all(new_dir).unwrap();
        sanitize_file(&video, &video_path).unwrap();
        sanitize_file(&audio, &audio_path).unwrap();

        Ok(Sources {
            video: video_path,
            audio: audio_path,
            output: output_path,
        })
    } else {
        Err(())
    }
}

fn extract_video_info(info: &PathBuf) -> VideoInfo {
    let mut file = File::open(info).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let info = VideoInfo::from_bytes(contents.as_bytes()).unwrap();
    info
}

fn sanitize_file(input_path: &PathBuf, output_path: &PathBuf) -> Result<(), ()> {
    let mut input = File::open(input_path).unwrap();

    let mut buffer = [0; 1];
    // NOTE: skip 0x30 (ASCII 0) bytes at the beginning of the file
    while input.read_exact(&mut buffer).is_ok() && buffer[0] == 0x30 {
        continue;
    }
    input.seek(SeekFrom::Current(-1)).unwrap();

    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
        .unwrap();

    io::copy(&mut input, &mut output).unwrap();

    Ok(())
}
