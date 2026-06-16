use core::f32;
use std::cmp::min;

use crate::FontMeta;
use crate::bar::{Bar, BarEl, BarInteraction};
use crate::colors::HIGHLIGHT_COLOR;
use crate::constants::{BARLINE_Y_SPACING, MUSIC_FONT};
use crate::note_or_rest::NoteOrRest;
use crate::pitch::{Pitch, PitchClass};
use iced::widget::Action;
use iced::widget::canvas::Style::Gradient;
use iced::widget::canvas::gradient::Linear;
use iced::widget::canvas::{self, Frame, LineCap, LineDash, LineJoin, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme, Vector, mouse};

use crate::Message;

const TOP_PADDIING: f32 = 30.;
// Padding in front of every bar before the first note
const PREAMBLE_WIDTH: f32 = 1.;
// const PREAMBLE_WIDTH: f32 = 6. * BARLINE_Y_SPACING;
const NOTE_Y_SPACING: f32 = BARLINE_Y_SPACING / 2.;

#[derive(Debug, Default)]
pub struct Staff {
    // The bars should be sorted by the order they come in the music
    pub bars: Vec<Bar>,
}

pub struct StaffEl {
    // The BarEls should be sorted by the order they come in the music and are drawn
    bars: Vec<BarEl>,
    cache: canvas::Cache,
    font: FontMeta,
    // Invariant: `width` should be the rendered width of `staff`
    width: f32,
}

impl StaffEl {
    pub fn new(staff: Staff, font: FontMeta) -> Self {
        let bars: Vec<BarEl> = staff
            .bars
            .into_iter()
            .scan(PREAMBLE_WIDTH, |x, bar| {
                let b = BarEl::new(bar, *x, &font);
                *x += b.get_width();
                Some(b)
            })
            .collect();

        let width = bars.last().map_or(0.0, |b| b.get_x() + b.get_width());

        StaffEl {
            bars,
            font: font,
            cache: canvas::Cache::default(),
            width,
        }
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn add_bar(self: &mut Self) {
        let new_bar = Bar::new(vec![NoteOrRest::new(None, 1)]);
        let new_el = BarEl::new(new_bar, self.width, &self.font);
        self.width += new_el.get_width();
        self.bars.push(new_el);
    }

    pub fn set_note(self: &mut Self, staff_index: &StaffIndex, pitch: Pitch) {
        let bar = &mut self.bars[staff_index.bar_index];
        let w = bar.get_width();
        bar.set_note(staff_index.note_index, pitch, &self.font);
        let dw = bar.get_width() - w;
        for i in staff_index.bar_index + 1..self.bars.len() {
            self.bars[i].translate_x(dw);
        }
        self.width += dw;
    }

    pub fn redraw(&mut self) {
        self.cache.clear();
    }
}

#[derive(Default)]
pub struct State {
    staff_interaction: StaffInteraction,
    hovering_new_bar: bool,
    new_bar_button_bounds: Rectangle,
}

#[derive(Debug, Default)]
enum StaffInteraction {
    #[default]
    None,
    // Hovering a note
    Hovering(StaffIndex),
    // Selected note at `NoteIndex` and mouse is hovering `Pitch`
    Selected(StaffIndex, Pitch),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StaffIndex {
    bar_index: usize,
    note_index: usize,
}

// Figure out which note the cursor is hovering over
//
// The cursor_pos should be given relative to the staff coordinates
// Returns the note that has the largest x value smaller than the cursor's x
// if the cursor is withing bounds of that note
fn get_hovering(cursor_pos: &Point, staff: &StaffEl) -> Option<StaffIndex> {
    let possible_bar_idx = min(
        staff.bars.partition_point(|bar| {
            bar.get_notes()
                .last()
                .map_or(true, |last| last.get_left_bound() < cursor_pos.x)
        }),
        staff.bars.len() - 1,
    );

    let idx = staff.bars[possible_bar_idx]
        .get_notes()
        .partition_point(|note| note.get_left_bound() < cursor_pos.x);

    if idx == 0 {
        // If every element in the possible bar is too far to the right
        // Then keep looking back for the next largest element
        for i in (0..possible_bar_idx).rev() {
            let notes = staff.bars[i].get_notes();
            let l = notes.len();
            if l != 0 && cursor_pos.x < notes[l - 1].get_right_bound() {
                return Some({
                    StaffIndex {
                        bar_index: i,
                        note_index: l - 1,
                    }
                });
            }
        }
    } else if staff.bars[possible_bar_idx].get_notes()[idx - 1].get_right_bound() >= cursor_pos.x {
        return Some(StaffIndex {
            bar_index: possible_bar_idx,
            note_index: idx - 1,
        });
    }

    return None;
}

fn y_offset_to_pitch(y: f32) -> Pitch {
    let note_space: f32 = 3. - y / NOTE_Y_SPACING;
    let mut pitch_class_num = (note_space % 7.).round() as i32;
    if pitch_class_num < 0 {
        pitch_class_num += 7;
    }
    let pitch_class = match pitch_class_num {
        0 => PitchClass::C,
        1 => PitchClass::D,
        2 => PitchClass::E,
        3 => PitchClass::F,
        4 => PitchClass::G,
        5 => PitchClass::A,
        6 => PitchClass::B,
        _ => panic!("Modular Arithmetic Error"),
    };
    let octave = ((note_space.round() / 7.).floor() + 5.) as u8;

    Pitch::new(pitch_class, octave)
}

fn draw_bar_lines(frame: &mut Frame, bounds: Rectangle, thickness: f32) {
    let spacing = bounds.height / 4.;
    for i in 0..5 {
        let y = i as f32 * spacing;
        let from = Point::new(bounds.x, y - thickness / 2.);
        // let to = Point::new(bounds.x + bounds.width, y);
        let path = canvas::Path::rectangle(from, iced::Size::new(bounds.width, thickness));
        frame.fill(&path, Color::BLACK);
    }
}

impl canvas::Program<Message> for StaffEl {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // TODO: Clean up update logic
        let new_bar_width = 4. * BARLINE_Y_SPACING;
        let new_bar_button_bounds = Rectangle {
            x: self.get_width(),
            y: 0.,
            width: new_bar_width,
            height: 4. * BARLINE_Y_SPACING,
        };
        state.new_bar_button_bounds = new_bar_button_bounds;

        match event {
            iced::Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(button) => {
                    match button {
                        mouse::Button::Left => {
                            if let Some(position) =
                                cursor.position_in(bounds + Vector::new(0., TOP_PADDIING))
                            {
                                if state.new_bar_button_bounds.contains(position) {
                                    return Some(canvas::Action::publish(Message::AddBar));
                                }

                                match state.staff_interaction {
                                    StaffInteraction::None => (),
                                    StaffInteraction::Hovering(staff_index) => {
                                        // TODO: Rerender
                                        state.staff_interaction = StaffInteraction::Selected(
                                            staff_index,
                                            y_offset_to_pitch(position.y),
                                        );
                                    }
                                    StaffInteraction::Selected(staff_index, pitch) => {
                                        state.staff_interaction = StaffInteraction::None;
                                        return Some(canvas::Action::publish(Message::SetNote(
                                            staff_index,
                                            pitch,
                                        )));
                                    }
                                }
                            }
                        }
                        _ => return None,
                    }
                    return None;
                }
                mouse::Event::CursorMoved { .. } => {
                    match cursor.position_in(bounds + Vector::new(0., TOP_PADDIING)) {
                        Some(position) => {
                            let mut should_rerender;

                            if state.new_bar_button_bounds.contains(position) {
                                should_rerender = !state.hovering_new_bar;
                                state.hovering_new_bar = true;
                            } else {
                                should_rerender = state.hovering_new_bar;
                                state.hovering_new_bar = false;
                            }

                            let hovering = get_hovering(&position, self);
                            match &state.staff_interaction {
                                StaffInteraction::None => {
                                    if let Some(staff_index) = hovering {
                                        should_rerender = true;
                                        state.staff_interaction =
                                            StaffInteraction::Hovering(staff_index);
                                    }
                                }
                                StaffInteraction::Hovering(staff_index) => match hovering {
                                    Some(hovering_index) => {
                                        if hovering_index != *staff_index {
                                            should_rerender = true;
                                            state.staff_interaction =
                                                StaffInteraction::Hovering(hovering_index);
                                        }
                                    }
                                    None => {
                                        should_rerender = true;
                                        state.staff_interaction = StaffInteraction::None;
                                    }
                                },
                                StaffInteraction::Selected(staff_index, pitch) => {
                                    let p = y_offset_to_pitch(position.y);
                                    if p != *pitch {
                                        state.staff_interaction =
                                            StaffInteraction::Selected(*staff_index, p);
                                        should_rerender = true;
                                    }
                                }
                            }

                            if should_rerender {
                                self.cache.clear();
                                Some(Action::request_redraw())
                            } else {
                                None
                            }
                        }
                        None => {
                            if state.hovering_new_bar {
                                state.hovering_new_bar = false;
                                self.cache.clear();
                                Some(Action::request_redraw())
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
            iced::Event::Keyboard(_event) => None,
            iced::Event::Window(_event) => None,
            iced::Event::Touch(_event) => None,
            iced::Event::InputMethod(_event) => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let _theme: &Theme = theme;
        let _cursor = cursor;
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.with_save(|f| {
                f.scale(1.);
                for i in 1..12 {
                    let t = canvas::Text {
                        size: (50 + 10 * i).into(),
                        position: Point::new(500. + 30. * i as f32, 1.),
                        shaping: iced::widget::text::Shaping::Basic,
                        font: self.font.font_iced,
                        .."\u{E0A3}".into()
                    };
                    t.draw_with(|path, color| {
                        f.fill(&path, Color::from_rgb(1., 0., 0.));
                    });
                }
            });

            frame.with_save(|f| {
                f.scale(0.5);
                for i in 1..12 {
                    let t = canvas::Text {
                        size: (100 + 10 * i).into(),
                        position: Point::new(1. + 30. * i as f32, 1.),
                        shaping: iced::widget::text::Shaping::Basic,
                        font: self.font.font_iced,
                        .."\u{E0A3}".into()
                    };
                    t.draw_with(|path, color| {
                        f.fill(&path, Color::from_rgb(1., 0., 0.));
                    });
                }
            });
            // frame.fill(
            //     &Path::circle(Point::new(50., 50.), 10.),
            //     Color::from_rgb(1., 0., 0.),
            // );

            frame.with_save(|f| {
                f.scale(BARLINE_Y_SPACING * 4.);
                f.translate(Vector::new(0., 2.));
                // f.fill(&Path::circle(Point::ORIGIN, 1.), Color::BLACK);

                let staff_bar_line_bounds = Rectangle {
                    x: 0.,
                    y: 0.,
                    width: self.get_width(),
                    height: 1.,
                };
                draw_bar_lines(
                    f,
                    staff_bar_line_bounds,
                    self.font.engraving_defaults.staff_line_thickness / 4.,
                    // Stroke::default().with_width(2.),
                    // .with_width(self.font.engraving_defaults.staff_line_thickness / 4.),
                );

                // new bar icon
                // let new_bar_stroke = Stroke {
                //     style: Gradient(
                //         (Linear::new(
                //             Point::new(state.new_bar_button_bounds.x, 0.),
                //             Point::new(
                //                 state.new_bar_button_bounds.x + state.new_bar_button_bounds.width,
                //                 0.,
                //             ),
                //         )
                //         .add_stop(0., Color::from_rgb(0.4, 0.4, 0.4))
                //         .add_stop(1., Color::from_rgb(0.9, 0.9, 0.9)))
                //         .into(),
                //     ),
                //     width: self.font.engraving_defaults.staff_line_thickness * BARLINE_Y_SPACING,
                //     line_cap: LineCap::default(),
                //     line_join: LineJoin::default(),
                //     line_dash: LineDash::default(),
                // };
                // draw_bar_lines(f, state.new_bar_button_bounds, new_bar_stroke);
                // f.fill(
                //     &Path::circle(
                //         state.new_bar_button_bounds.center(),
                //         BARLINE_Y_SPACING * 0.8,
                //     ),
                //     Color::from_rgb(1., 1., 1.),
                // );
                // let plus_path = Path::new(|b| {
                //     let r = BARLINE_Y_SPACING * 0.4;
                //     b.move_to(state.new_bar_button_bounds.center() + Vector::new(-r, 0.));
                //     b.line_to(state.new_bar_button_bounds.center() + Vector::new(r, 0.));
                //
                //     b.move_to(state.new_bar_button_bounds.center() + Vector::new(0., -r));
                //     b.line_to(state.new_bar_button_bounds.center() + Vector::new(0., r));
                // });
                // f.stroke(
                //     &plus_path,
                //     Stroke::default()
                //         .with_width(3.)
                //         .with_color(if state.hovering_new_bar {
                //             HIGHLIGHT_COLOR
                //         } else {
                //             Color::from_rgb(0.4, 0.4, 0.4)
                //         })
                //         .with_line_cap(LineCap::Round),
                // );

                for (i, bar) in self.bars.iter().enumerate() {
                    let bar_interaction = match &state.staff_interaction {
                        StaffInteraction::None => BarInteraction::None,
                        StaffInteraction::Hovering(staff_index) => {
                            if staff_index.bar_index == i {
                                BarInteraction::Hovering(staff_index.note_index)
                            } else {
                                BarInteraction::None
                            }
                        }
                        StaffInteraction::Selected(staff_idx, selected_pitch) => {
                            if staff_idx.bar_index == i {
                                BarInteraction::Selected(staff_idx.note_index, *selected_pitch)
                            } else {
                                BarInteraction::None
                            }
                        }
                    };
                    bar.draw(f, &bar_interaction, &self.font);
                }
            });

            // let mut my_text: canvas::Text = "\u{E050}".into();
            // my_text.font = MUSIC_FONT;
            // my_text.size = (4. * BARLINE_Y_SPACING).into();
            // my_text.align_y = iced::alignment::Vertical::Center;
            // my_text.position = Point::new(0., 3. * BARLINE_Y_SPACING);
            //
            // my_text.draw_with(|path, color| {
            //     frame.fill(&path, color);
            // });
        });
        vec![geom]
    }
}
