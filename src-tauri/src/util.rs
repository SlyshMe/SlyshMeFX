use std::{thread, time::{Duration, Instant}, vec};

use audioviz::spectrum::{config::StreamConfig, stream::Stream, Frequency};
use fast_math::log2;
use tauri::{AppHandle, Emitter};
use wasapi::{initialize_mta, AudioClient, Direction, ShareMode, WaveFormat};

use crate::{structs::VisualiserType, FrequencyInterval};



pub fn makeDistribution(data: &[Frequency], resolution: usize) -> Vec<FrequencyInterval> {
    if resolution == 0 {
        return vec![];
    }

    let visualiserType = crate::VISUALISER_CONFIG.read().unwrap().visualiserType;
    match visualiserType {
        // Group by frequency
        VisualiserType::Linear1 => {
            let mut last = 0.;
            let freqStep = 20_000. / resolution as f32;
            let mut v: Vec<Vec<f32>> = vec![vec![]];

            for i in data {
                if i.freq < last + freqStep {
                    v.last_mut().unwrap().push(i.volume);
                } else {
                    v.push(vec![i.volume]);
                    last += freqStep;
                }
            }

            for _ in 0..resolution - v.len() {
                v.push(vec![]);
            }

            v.iter()
                .enumerate()
                .map(|(i, interval)| FrequencyInterval {
                    index: i as u16,
                    volume: interval.iter().sum::<f32>() / interval.len() as f32,
                })
                .collect()
        },
        // Group by amount
        VisualiserType::Linear2 => {
            let mut result = Vec::with_capacity(resolution);
            
            let n = data.len();
            let step = n as f32 / resolution as f32;

            for i in 0..resolution {
                let start_f = (i as f32) * step;
                let end_f = ((i + 1) as f32) * step;
                let start = start_f.floor() as usize;
                let end = end_f.floor() as usize;

                let start = start.min(n);
                let end = end.min(n);

                let volume = if start < end {
                    let slice = &data[start..end];
                    let sum: f32 = slice.iter().map(|f| f.volume).sum();

                    sum / (slice.len() as f32)
                } else { 0.0 };

                result.push(FrequencyInterval {
                    index: i as u16,
                    volume: volume,
                });
            }

            result
        },
        // Group by frequency, logarithmic
        VisualiserType::Log => {
            let mut intervals = vec![FrequencyInterval { index: 0, volume: 0. }; resolution];

            let min = log2(20.);
            let max = log2(20_000.);
            let range = max - min;

            let mut lastIndex = 0;
            let mut lastVolume = 0.;
            let mut lastCount = 0;
            for i in data {
                let normalisedLogFreq = (log2(i.freq) - min) / range;
                let mut index = (normalisedLogFreq * (resolution as f32 - 0.001)).floor() as usize;
                if index >= resolution {
                    index = resolution - 1;
                }

                if index == lastIndex {
                    lastVolume += i.volume;
                    lastCount += 1;
                } else {
                    intervals[lastIndex].index = lastIndex as u16;
                    intervals[lastIndex].volume = lastVolume / lastCount as f32;

                    lastIndex = index;
                    lastVolume = i.volume;
                    lastCount = 1;
                }
                
                intervals[lastIndex].index = lastIndex as u16;
                intervals[lastIndex].volume = lastVolume / lastCount as f32;
            }

            intervals
        }
    }
}

pub fn audioCapture(appHandle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // WasApi setup
    let _ = initialize_mta();

    let pid = std::process::id();
    let mut client = AudioClient::new_application_loopback_client(pid, false)?;
    let waveform = WaveFormat::new(32, 32, &wasapi::SampleType::Float, 48_000, 2, None);

    client.initialize_client(
        &waveform,
        i64::from(waveform.get_avgbytespersec()) * (15 / 1_000), // 15 ms
        &Direction::Capture,
        &ShareMode::Shared,
        true,
    )?;
    client.start_stream()?;

    let capClient = client.get_audiocaptureclient()?;
    let mut buffer =
        vec![0u8; (waveform.get_blockalign() * waveform.get_samplespersec() * 5) as usize];
    let mut sampleBuffer = Vec::<f32>::new();

    // Spectrum analysis
    let mut spec = Stream::new(StreamConfig::default());
    let mut last = Instant::now();
    let target = Duration::from_millis(15);

    loop {
        thread::sleep(Duration::from_millis(5));

        let (frames, flags) = capClient.read_from_device(&mut buffer)?;
        if frames > 0 {
            let bytes = &buffer[..(frames * waveform.get_blockalign()) as usize];
            let samples = unsafe {
                std::slice::from_raw_parts(
                    bytes.as_ptr() as *const f32,
                    bytes.len() / std::mem::size_of::<f32>(),
                )
            };

            for chunk in samples.chunks(waveform.get_nchannels() as usize) {
                sampleBuffer.push(chunk[0]);
            }
        }

        if last.elapsed() >= target {
            spec.push_data(sampleBuffer.clone());
            spec.update();

            let freqs = spec.get_frequencies();

            if freqs.len() > 0 {
                let magnitudes: Vec<String> = makeDistribution(&freqs[0], crate::VISUALISER_CONFIG.read().unwrap().resolution.into())
                // let magnitudes: Vec<String> = makeDistribution(&freqs[0], 128)
                    .iter()
                    .map(|f| format!(r#"{{ "index": {}, "volume": {} }}"#, f.index, f.volume))
                    .collect();

                if let Err(e) = appHandle.emit("spectrum", magnitudes.clone()) {
                    eprintln!("Failed to emit audio-spectrum event: {}", e);
                }

                sampleBuffer.clear();
                last = Instant::now();
            }
        }

        if flags.data_discontinuity {
            eprintln!("Discontinuity detected – reset state.");
            sampleBuffer.clear();
        }
    }
}
