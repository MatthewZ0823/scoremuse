use std::{fs::File, io::Write, num::NonZero, sync::Arc};

use rodio::{ChannelCount, Decoder, Source};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::staff::StaffEl;

pub fn synthesize_staff() {
    // Load the SoundFont.
    let mut sf2 = File::open("soundfonts/TimGM6mb.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    // Create the synthesizer.
    let settings = SynthesizerSettings::new(44100);
    let mut synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    // The output buffer (5 seconds).
    let sample_count = (5 * settings.sample_rate) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Play some notes (middle C, E, G).
    synthesizer.process_midi_message(0, 0xB0, 0x07, 0x7F);
    synthesizer.note_on(0, 60, 100);
    synthesizer.note_on(0, 64, 100);
    synthesizer.note_on(0, 67, 100);

    // Render the waveform.
    synthesizer.render(&mut left[..], &mut right[..]);

    synthesizer.note_off_all(true);
    // synthesizer.note_on(0, 61, 100);
    // synthesizer.note_on(0, 65, 100);
    // synthesizer.note_on(0, 68, 100);

    // Render the waveform.
    synthesizer.render(&mut left[44100..], &mut right[44100..]);

    synthesizer.process_midi_message(0, 0xB0, 0x07, 0x3F);
    synthesizer.note_on(0, 60, 100);
    synthesizer.note_on(0, 64, 100);
    synthesizer.note_on(0, 67, 100);
    synthesizer.render(&mut left[2 * 44100..], &mut right[2 * 44100..]);

    synthesizer.note_off_all(false);
    synthesizer.render(&mut left[3 * 44100..], &mut right[3 * 44100..]);

    // // Write the waveform to the file.
    // write_pcm(&left[..], &right[..], "simple_chord.pcm");

    // Get an OS-Sink handle to the default physical sound device.
    // Note that the playback stops when the handle is dropped.//!
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    // let player = rodio::Player::connect_new(&handle.mixer());

    let source = MySource::new(left, right);

    // Play the sound directly on the device
    handle.mixer().add(source);

    std::thread::sleep(std::time::Duration::from_secs(5));
}

/// Requires the left and right to have the same length
struct MySource {
    left: Vec<f32>,
    right: Vec<f32>,
    i: usize,
}

impl MySource {
    fn new(left: Vec<f32>, right: Vec<f32>) -> Self {
        MySource { left, right, i: 0 }
    }
}

impl Source for MySource {
    fn current_span_len(&self) -> Option<usize> {
        Some(2 * self.left.len() - self.i)
    }

    fn channels(&self) -> rodio::ChannelCount {
        unsafe { NonZero::new_unchecked(2 as u16).into() }
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        // TODO: Stop hard coding 44100
        unsafe { NonZero::new_unchecked(44100 as u16).into() }
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(
            (self.left.len() as f32 / 44100. * 1000.).round() as u64,
        ))
    }
}

impl Iterator for MySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 2 * self.left.len() {
            return None;
        }

        let ret = if self.i % 2 == 0 {
            Some(self.left[self.i / 2])
        } else {
            Some(self.right[self.i / 2])
        };
        self.i += 1;
        ret
    }
}

fn write_pcm(left: &[f32], right: &[f32], path: &str) {
    let mut max: f32 = 0_f32;
    for t in 0..left.len() {
        if left[t].abs() > max {
            max = left[t].abs();
        }
        if right[t].abs() > max {
            max = right[t].abs();
        }
    }
    let a = 0.99_f32 / max;

    let mut buf: Vec<u8> = vec![0; 4 * left.len()];
    for t in 0..left.len() {
        let left_i16 = (a * left[t] * 32768_f32) as i16;
        let right_i16 = (a * right[t] * 32768_f32) as i16;

        let offset = 4 * t;
        buf[offset] = left_i16 as u8;
        buf[offset + 1] = (left_i16 >> 8) as u8;
        buf[offset + 2] = right_i16 as u8;
        buf[offset + 3] = (right_i16 >> 8) as u8;
    }

    let mut pcm = File::create(path).unwrap();
    pcm.write_all(&buf[..]).unwrap();
}
