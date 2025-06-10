use std::{fs::File, path::Path};

use symphonia::{core::{audio::{AudioBufferRef, Signal}, codecs::CODEC_TYPE_NULL, io::MediaSourceStream}, default::{get_codecs, get_probe}};

/// 
/// Creates a waveform representation using u8s, where 0 is quiet and 255 is loud.
///
/// # Parameters:
/// - `file`: The file path
/// - `sample_count`: The number of samples to return
///
/// # Returns:
/// - a vector of u8s representing the waveform
/// - a string explaining why the function failed
///
pub fn get_sampled_waveform(file: &str, sample_count: usize) -> Result<Vec<u8>, String> {
    
    let path = Path::new(&file);
    match path.try_exists() {
        Ok(exists) => {
            if !exists {
                return Err("file did not exist".to_string());
            }
        },
        Err(err) => {
            return Err(err.to_string());
        }
    }

    let audio_file = match File::open(path) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mss = MediaSourceStream::new(Box::new(audio_file), Default::default());

    let mut probed_audio = match get_probe().format(&Default::default(), mss, &Default::default(), &Default::default()) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let track = match probed_audio.format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL) {
        Some(val) => val,
        None => {
            return Err("track had no codec".to_string());
        }
    };

    let mut decoder = match get_codecs().make(&track.codec_params, &Default::default()) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut all_samples: Vec<f32> = Vec::new();
    while let Ok(packet) = probed_audio.format.next_packet() {
        let decoded_packet = match decoder.decode(&packet) {
            Ok(val) => val,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        let channels = decoded_packet.spec().channels.count();
        if channels == 0 {
            return Err("audio had no channels".to_string());
        }

        let mut samples = samples_as_f32(&decoded_packet, 0); // first take channel 0
        for c in 1..channels {
            let next_channel_samples = samples_as_f32(&decoded_packet, c);
            for i in 0..next_channel_samples.len() {
                samples[i] = (samples[i] + next_channel_samples[i]) * (c as f32 / (c as f32 + 1.));
            }
        }

        // here we normalise -1 <-> 1 to 0 <-> 1 with abs
        all_samples.append(&mut samples);
    }

    let sample_group_size = (all_samples.len() as f64 / sample_count as f64).ceil() as usize;
    let mut means: Vec<f32> = Vec::new();

    for i in 0..sample_count {
        let start = i * sample_group_size;
        let end = ((i + 1) * sample_group_size).min(all_samples.len());
        if start < end {
            let section: &[f32] = &all_samples[start..end];
            let mean = section.iter().copied().sum::<f32>() / section.len() as f32;
            means.push(mean);
        }
    }

    Ok(means.iter().map(|s| (s * 255.).floor() as u8).collect())
}

/// 
/// Decodes each audio channel from any type to an f32
///
/// # Parameters:
/// - `buf`: A reference to an audio buffer
/// - `chan`: The channel number of he audio buffer
///
/// # Returns:
/// - a vector of f32 values
///
fn samples_as_f32(buf: &AudioBufferRef, chan: usize) -> Vec<f32> {
    match buf {
        AudioBufferRef::U8(buf) => buf.chan(chan).iter().map(|s|  *s as f32).collect(),
        AudioBufferRef::S16(buf) => buf.chan(chan).iter().map(|s| *s as f32).collect(),
        AudioBufferRef::U16(buf) => buf.chan(chan).iter().map(|s| *s as f32).collect(),
        AudioBufferRef::S32(buf) => buf.chan(chan).iter().map(|s| *s as f32).collect(),
        AudioBufferRef::F64(buf) => buf.chan(chan).iter().map(|s| *s as f32).collect(),
        AudioBufferRef::F32(buf) => buf.chan(chan).to_vec(),
        _ => Vec::new(), // or handle error
    }
}
