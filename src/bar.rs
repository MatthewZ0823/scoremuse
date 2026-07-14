use crate::constants::{BARLINE_LEFT_PADDING, STANDARD_STAFF_SPACING};
use crate::font;
use crate::note_or_rest::{BaseDuration, NoteInteraction, NoteOrRest, NoteOrRestEl};
use crate::pitch::Pitch;
use iced::{Color, Size};
use iced::{
    Point,
    widget::canvas::{Frame, Path},
};
use num_traits::Pow;

#[derive(Debug)]
pub struct Bar {
    // notes should be ordered by start
    notes: Vec<NoteOrRest>,
}

impl Bar {
    pub fn new(notes: Vec<NoteOrRest>) -> Self {
        Bar { notes }
    }
}

pub struct BarEl {
    /// notes should be ordered as they are in the music and as they are drawn
    notes: Vec<NoteOrRestEl>,
    /// The x-position of the left bound
    x: f32,
    /// Total width, including padding
    width: f32,
}

pub enum BarInteraction {
    None,
    // Hovering a note at index `usize`
    Hovering(usize),
    // Selected a note at index and mouse is hovering pitch
    Selected(usize, Pitch),
}

impl BarEl {
    // `x` is the left bound of the bar
    pub fn new(bar: Bar, x: f32, font: &font::FontMeta) -> Self {
        let mut bar = BarEl {
            notes: bar
                .notes
                .into_iter()
                .map(|note| NoteOrRestEl::new(note, 0., font))
                .collect(),
            width: 0.,
            x,
        };
        bar.fix_layout(font);
        bar
    }

    pub fn get_width(self: &Self) -> f32 {
        self.width
    }

    pub fn draw(
        self: &Self,
        frame: &mut Frame,
        bar_interaction: &BarInteraction,
        font: &font::FontMeta,
    ) {
        for (i, note) in self.notes.iter().enumerate() {
            let note_interaction = match bar_interaction {
                BarInteraction::None => NoteInteraction::None,
                BarInteraction::Hovering(hovering_idx) => {
                    if *hovering_idx == i {
                        NoteInteraction::Hovering
                    } else {
                        NoteInteraction::None
                    }
                }
                BarInteraction::Selected(selected_idx, pitch) => {
                    if *selected_idx == i {
                        NoteInteraction::Selected(*pitch)
                    } else {
                        NoteInteraction::None
                    }
                }
            };
            note.draw(frame, &note_interaction, font);
        }

        let last_note = self.notes.last().expect("Bar should not be empty");
        let barline_x = last_note.get_right_bound();
        let barline_path = Path::rectangle(
            Point::new(barline_x, 0.),
            Size::new(
                font.barlines_meta.thin_thickness,
                4. * STANDARD_STAFF_SPACING,
            ),
        );
        frame.fill(&barline_path, Color::BLACK);
    }

    pub fn get_notes(&self) -> &Vec<NoteOrRestEl> {
        &self.notes
    }

    /// Setting pitch to None changes the note to arest
    /// May change the layout of the bar
    pub fn set_note_pitch(
        &mut self,
        note_index: usize,
        pitch: Option<Pitch>,
        font: &font::FontMeta,
    ) {
        self.notes[note_index].set_pitch(pitch, font);
        self.fix_layout(font);
    }

    /// May change the layout of the bar
    ///
    /// TODO: Make this work with other time signatures
    pub fn set_note_base_duration(
        &mut self,
        note_index: usize,
        base_duration: BaseDuration,
        font: &font::FontMeta,
    ) {
        // If there aren't enough beats in the bar to change this note duration, then don't do
        // anything
        let beats_after = self.notes[note_index..]
            .iter()
            .map(|n| n.get_base_duration().get_duration_beats())
            .sum::<f32>();
        if beats_after < base_duration.get_duration_beats() {
            return;
        }

        // If we need to add back some notes/rests after removing too many beats
        // Then we will need to keep track of these
        let mut replenish_pitch = None;
        let mut replenish_beats: f32;

        let prev_base_duration = self.notes[note_index].get_base_duration();
        self.notes[note_index].set_base_duration(base_duration, font);

        if base_duration <= prev_base_duration {
            // Shortening
            replenish_beats =
                prev_base_duration.get_duration_beats() - base_duration.get_duration_beats();
        } else {
            // Lengthening
            // Keep removing notes until we have enough space
            let mut extra_duration =
                base_duration.get_duration_beats() - prev_base_duration.get_duration_beats();
            let i = note_index + 1;

            while extra_duration > 0. {
                replenish_pitch = self.notes[i].get_pitch();
                extra_duration -= self.notes[i].get_base_duration().get_duration_beats();
                self.notes.remove(i);
            }

            // In case we remove too many beats
            replenish_beats = -extra_duration;
        }

        // Replenish the notes if we removed too many beats
        let mut replenish_i: usize = note_index + 1;
        let mut starting_beat: f32 = self.notes[..replenish_i]
            .iter()
            .map(|note| note.get_duration_beats())
            .sum();

        while replenish_beats > 0. {
            // Find the log of the largest beat that can fit
            let mut k: i16 = replenish_beats.log2().floor() as i16;

            // Keep decrementing k until the note fits nicely
            // Don't need to worry about floating point precision
            while starting_beat.rem_euclid(2_f32.pow(k)) != 0. {
                k -= 1;
            }

            let replenish_note_beat = 2_f32.pow(k);
            self.notes.insert(
                replenish_i,
                NoteOrRestEl::new(
                    NoteOrRest::new(
                        replenish_pitch,
                        BaseDuration::from_duration_beats(replenish_note_beat),
                    ),
                    0.,
                    font,
                ),
            );
            replenish_i += 1;
            replenish_beats -= replenish_note_beat;
            starting_beat += replenish_note_beat;
        }

        self.fix_layout(font);
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    // Translates the bar and its notes
    pub fn translate_x(&mut self, dx: f32) {
        self.x += dx;
        for note in &mut self.notes {
            note.translate_x(dx);
        }
    }

    /// Fix the note positions and the width of the bar after changing note at `note_index`
    // fn fix_layout(&mut self, note_index: usize, f: impl FnOnce(&mut NoteOrRestEl)) {
    //     let note = &mut self.notes[note_index];
    //     let w = note.get_width();
    //     f(note);
    //     let d_width = note.get_width() - w;
    //
    //     for i in (note_index + 1)..self.notes.len() {
    //         self.notes[i].translate_x(d_width);
    //     }
    //     self.width += d_width;
    // }

    /// Fix the note positions and the width of the bar
    fn fix_layout(&mut self, font: &font::FontMeta) {
        let mut x_ = self.get_x() + BARLINE_LEFT_PADDING;
        for note in self.notes.iter_mut() {
            note.set_x(x_);
            x_ += note.get_width();
        }

        self.width = x_ - self.get_x() + font.barlines_meta.single_advance_width;
    }
}
