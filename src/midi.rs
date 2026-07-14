use std::time::Duration;

use rustysynth::Synthesizer;

#[derive(Clone)]
pub struct MidiMessage {
    pub channel: i32,
    pub command: i32,
    pub data1: i32,
    pub data2: i32,
}

#[derive(Clone)]
pub struct MidiMessageTimed {
    /// The midi message
    pub midi_message: MidiMessage,
    /// How long until the next midi message
    pub duration: Duration,
}

impl MidiMessage {
    /// Construct a note on event message
    pub fn note_on(channel: i32, key: i32, velocity: i32) -> Self {
        MidiMessage {
            channel,
            command: 0x90,
            data1: key,
            data2: velocity,
        }
    }

    /// Constructs a message that stops all the notes in the specified channel
    /// If `immediate` then notes will stop without the release sound
    pub fn note_off_channel(channel: i32, immediate: bool) -> Self {
        MidiMessage {
            channel,
            command: 0xB0,
            data1: if immediate { 0x78 } else { 0x7B },
            data2: 0,
        }
    }

    pub fn process_on_synth(&self, synthesizer: &mut Synthesizer) {
        synthesizer.process_midi_message(self.channel, self.command, self.data1, self.data2);
    }
}

impl MidiMessageTimed {
    pub fn new(midi_message: MidiMessage, duration: Duration) -> Self {
        MidiMessageTimed {
            midi_message,
            duration,
        }
    }
}
