use std::{fs::File, num::NonZero, sync::Arc, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::futures::{FutureExt, SinkExt, Stream, StreamExt, channel::mpsc};
use rodio::{MixerDeviceSink, Source};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::{AudioCommand, AudioEvent, constants::SAMPLE_RATE, midi::MidiMessageTimed};

/// Play the `midi_messages` on the device associated with `audio_handle`
///
/// Playing will stop if the sender connected with `command_receiver` is dropped
// TODO: Make it so we can update midi_messages
pub fn play_midi(
    audio_handle: Arc<MixerDeviceSink>,
    midi_messages: Vec<MidiMessageTimed>,
    mut command_receiver: mpsc::Receiver<AudioCommand>,
) -> impl Stream<Item = AudioEvent> {
    // Load the SoundFont.
    let mut sf2 = File::open("soundfonts/TimGM6mb.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    // Create the synthesizer.
    let settings = SynthesizerSettings::new(SAMPLE_RATE as i32);
    let mut synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    // The output buffer
    let total_duration: Duration = midi_messages
        .iter()
        .map(|midi_message| midi_message.duration)
        .sum();
    let sample_count =
        (total_duration.as_millis() as f32 / 1000. * settings.sample_rate as f32).ceil() as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // TODO: Synthesis takes a lot of time, don't load all at once
    {
        let mut time: usize = 0; // In samples
        for midi_message in midi_messages {
            let d_time =
                (midi_message.duration.as_millis() as f32 / 1000. * SAMPLE_RATE as f32) as usize;

            midi_message.midi_message.process_on_synth(&mut synthesizer);
            synthesizer.render(&mut left[time..], &mut right[time..]);

            time += d_time;
        }
    }

    let mut source = MySource::new(left, right);

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    // TODO: Support other configs maybe
    let supported_config = supported_configs_range
        .find(|config| config.sample_format() == cpal::SampleFormat::I16 && config.channels() == 2)
        .expect("config not supported")
        .with_sample_rate(SAMPLE_RATE);

    // TODO: better errors
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let config = supported_config.into();
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    let next = source.next();
                    match next {
                        Some(s) => *sample = (s * (i16::MAX as f32)) as i16,
                        None => {
                            *sample = 0;
                        }
                    }
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    stream.play();

    // let player = rodio::Player::connect_new(audio_handle.mixer());
    // player.append(source);

    iced::stream::channel::<AudioEvent>(32, async move |mut output| {
        loop {
            // let remaining_time = total_duration - player.get_pos();
            let remaining_time = total_duration;
            iced::futures::select! {
                _ = tokio::time::sleep(remaining_time).fuse() => {
                    match output.send(AudioEvent::PlaybackDone).await {
                        Ok(_) => break,
                        Err(_) => panic!("Something went wrong with the audio event stream"),
                    }
                },
                command = command_receiver.next() => {
                    match command {
                        Some(command) => {
                            match command {
                                AudioCommand::Resume => stream.play(),
                                AudioCommand::Pause => stream.pause(),
                            };
                        },
                        None => break,
                    }
                }
            }
        }
    })
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
        unsafe { NonZero::new_unchecked(SAMPLE_RATE as u16).into() }
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(
            (self.left.len() as f32 / SAMPLE_RATE as f32 * 1000.).round() as u64,
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
