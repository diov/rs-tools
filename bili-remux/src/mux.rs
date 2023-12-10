use std::{fs::File, io::BufReader, path::Path};

use mp4::{AacConfig, AvcConfig, MediaConfig, Mp4Config, TrackConfig, TrackType};

pub fn remux<P: AsRef<Path>>(video: &P, audio: &P, output: &P) -> Result<(), ()> {
    let mut video_mp4 = mp4_reader(video).unwrap();
    let mut audio_mp4 = mp4_reader(audio).unwrap();
    // Get mp4 config from video file
    let config = Mp4Config {
        major_brand: video_mp4.major_brand().to_owned(),
        minor_version: video_mp4.minor_version(),
        compatible_brands: video_mp4.compatible_brands().to_vec(),
        timescale: video_mp4.timescale(),
    };

    let dst_file = File::create(output).unwrap();
    let mut writer = mp4::Mp4Writer::write_start(dst_file, &config).unwrap();

    // Add video track
    let video_track = video_mp4.tracks().get(&1).unwrap();
    let avc_conf = MediaConfig::AvcConfig(AvcConfig {
        width: video_track.width(),
        height: video_track.height(),
        seq_param_set: video_track.sequence_parameter_set().unwrap().to_vec(),
        pic_param_set: video_track.picture_parameter_set().unwrap().to_vec(),
    });
    let video_config = TrackConfig {
        track_type: TrackType::Video,
        timescale: video_track.timescale(),
        language: video_track.language().to_owned(),
        media_conf: avc_conf,
    };
    writer.add_track(&video_config).unwrap();

    // Add audio track
    let audio_track = audio_mp4.tracks().get(&2).unwrap();
    let aac_conf = MediaConfig::AacConfig(AacConfig {
        bitrate: audio_track.bitrate(),
        profile: audio_track.audio_profile().unwrap(),
        freq_index: audio_track.sample_freq_index().unwrap(),
        chan_conf: audio_track.channel_config().unwrap(),
    });
    let audio_config = TrackConfig {
        track_type: TrackType::Audio,
        timescale: audio_track.timescale(),
        language: audio_track.language().to_owned(),
        media_conf: aac_conf,
    };
    writer.add_track(&audio_config).unwrap();

    // Write sample to writer
    let video_track_id = video_track.track_id();
    for sample_idx in 0..video_mp4.sample_count(video_track_id).unwrap() {
        let sample = video_mp4
            .read_sample(video_track_id, sample_idx)
            .unwrap()
            .unwrap();
        writer.write_sample(video_track_id, &sample).unwrap();
    }
    let audio_track_id = audio_track.track_id();
    for sample_idx in 0..audio_mp4.sample_count(audio_track_id).unwrap() {
        let sample = audio_mp4
            .read_sample(audio_track_id, sample_idx)
            .unwrap()
            .unwrap();
        writer.write_sample(audio_track_id, &sample).unwrap();
    }

    writer.write_end().unwrap();

    Ok(())
}

fn mp4_reader<P: AsRef<Path>>(filename: &P) -> Result<mp4::Mp4Reader<BufReader<File>>, ()> {
    let f = File::open(filename).unwrap();
    let size = f.metadata().unwrap().len();
    let reader = BufReader::new(f);
    mp4::Mp4Reader::read_header(reader, size).map_err(|_| ())
}
