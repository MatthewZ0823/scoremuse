use crate::constants::{BARLINE_LEFT_PADDING, STANDARD_STAFF_SPACING};
use crate::font;
use crate::note_or_rest::{BaseDuration, NoteInteraction, NoteOrRest, NoteOrRestEl};
use crate::pitch::Pitch;
use iced::widget::canvas::Stroke;
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

    /// May change the layout of the bar
    pub fn set_note_pitch(&mut self, note_index: usize, pitch: Pitch, font: &font::FontMeta) {
        self.notes[note_index].set_pitch(pitch, font);
        self.fix_layout(font);
    }

    /// May change the layout of the bar
    ///
    /// TODO: Make this work with other time signatures
    /// TODO: Make this work with dotted notes and arbitrary(ish) duration notes
    pub fn set_note_base_duration(
        &mut self,
        note_index: usize,
        base_duration: BaseDuration,
        font: &font::FontMeta,
    ) {
        let beats_after = self.notes[note_index..]
            .iter()
            .map(|n| n.get_base_duration().get_duration_beats())
            .sum::<f32>();

        if beats_after >= base_duration.get_duration_beats() {
            let prev_base_duration = self.notes[note_index].get_base_duration();
            self.notes[note_index].set_base_duration(base_duration, font);

            if base_duration <= prev_base_duration {
                // Shortening
                let num_filler = 2.pow(base_duration.0 - prev_base_duration.0) - 1;

                for _ in 0..num_filler {
                    self.notes.insert(
                        note_index + 1,
                        NoteOrRestEl::new(NoteOrRest::new(None, base_duration), 0., font),
                    );
                }
            } else {
                dbg!("Lengthening");
                // Lengthening
                let mut extra_duration =
                    base_duration.get_duration_beats() - prev_base_duration.get_duration_beats();
                let i = note_index + 1;
                let mut replenish_pitch = None;

                // Remove enough notes after
                while extra_duration > 0. {
                    replenish_pitch = self.notes[i].get_pitch();
                    extra_duration -= self.notes[i].get_base_duration().get_duration_beats();
                    self.notes.remove(i);
                }

                // Replenish the notes if we removed too many beats
                let mut replenish_i = i;
                let mut replenish = -extra_duration;
                let mut starting_beat: f32 = self
                    .notes
                    .iter()
                    .map(|note| note.get_duration_beats())
                    .sum();
                while replenish > 0. {
                    // We want k to be the largest number where `starting_beat` = n * 2^k
                    // for some integer n
                    let mut k: i16 = 2;
                    while k > -8 {
                        let n: f32 = (starting_beat as f32) / (2_f32.pow(k as f32));
                        if n.trunc() == n {
                            break;
                        } else {
                            k -= 1;
                        }
                    }
                    if k == -8 {
                        panic!("Durations shorter than 128th not yet implemented")
                    }

                    // k should now satisfy the above property
                    dbg!(k);
                    dbg!(replenish);
                    let k_beats = 2_f32.pow(k);
                    if k_beats <= replenish {
                        replenish -= k_beats;
                        self.notes.insert(
                            replenish_i,
                            NoteOrRestEl::new(
                                NoteOrRest::new(
                                    replenish_pitch,
                                    BaseDuration::from_duration_beats(k_beats),
                                ),
                                0.,
                                font,
                            ),
                        );
                        replenish_i += 1;
                        starting_beat += k_beats;
                    } else {
                        // We want j to be the largest integer where 2^j <= replenish_beats
                        let j = replenish.log2().floor();
                        let j_beats = 2_f32.pow(j);
                        replenish -= j_beats;
                        self.notes.insert(
                            replenish_i,
                            NoteOrRestEl::new(
                                NoteOrRest::new(
                                    replenish_pitch,
                                    BaseDuration::from_duration_beats(k_beats),
                                ),
                                0.,
                                font,
                            ),
                        );
                        replenish_i += 1;
                        starting_beat += j_beats;
                    }
                }
            }

            self.fix_layout(font);
        }
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
