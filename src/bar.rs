use crate::constants::BARLINE_Y_SPACING;
use crate::font;
use crate::note_or_rest::{NoteInteraction, NoteOrRest, NoteOrRestEl};
use crate::pitch::Pitch;
use iced::widget::canvas;
use iced::{
    Point,
    widget::canvas::{Frame, Path},
};

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
    // notes should be ordered as they are in the music and as they are drawn
    notes: Vec<NoteOrRestEl>,
    // `x` is the left bound of the bar
    x: f32,
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
        let mut note_els: Vec<NoteOrRestEl> = vec![];
        let mut x_ = x;
        for note in bar.notes.into_iter() {
            let n = NoteOrRestEl::new(note, x_, font);
            x_ += n.get_width();
            note_els.push(n);
        }

        BarEl {
            notes: note_els,
            width: x_ - x,
            x,
        }
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

        let barline_x = self.x + self.width;
        let barline_path = Path::line(
            Point::new(barline_x, 0.),
            Point::new(barline_x, 4. * BARLINE_Y_SPACING),
        );
        frame.stroke(&barline_path, canvas::Stroke::default());
    }

    pub fn get_notes(&self) -> &Vec<NoteOrRestEl> {
        &self.notes
    }

    pub fn set_note(&mut self, note_index: usize, pitch: Pitch, font: &font::FontMeta) {
        let note = &mut self.notes[note_index];
        let w = note.get_width();
        note.set_pitch(pitch, font);
        let dw = note.get_width() - w;

        for i in (note_index + 1)..self.notes.len() {
            self.notes[i].translate_x(dw);
        }
        self.width += dw;
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
}
