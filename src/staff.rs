use core::f32;
use std::cmp::min;
use std::time::Duration;

use crate::bar::{Bar, BarEl, BarInteraction};
use crate::constants::STANDARD_STAFF_SPACING;
use crate::midi::{MidiMessage, MidiMessageTimed};
use crate::note_or_rest::{BaseDuration, NoteOrRest};
use crate::pitch::Pitch;
use crate::{FontMeta, ScoreEditingMessage, note_or_rest};
use iced::widget::Action;
use iced::widget::canvas::{self, Frame};
use iced::{Color, Point, Rectangle, Renderer, Theme, Vector, mouse};

// Padding in front first bar
const PREAMBLE_WIDTH: f32 = 100.;
// const PREAMBLE_WIDTH: f32 = 6. * BARLINE_Y_SPACING;

#[derive(Debug, Default)]
pub struct Staff {
    // The bars should be sorted by the order they come in the music
    pub bars: Vec<Bar>,
}

pub struct StaffEl {
    // The BarEls should be sorted by the order they come in the music and are drawn
    // Each BarEl should not be empty, ie. they should have some notes/rests
    bars: Vec<BarEl>,
    cache: canvas::Cache,
    font: FontMeta,
    // Invariant: `width` should be the rendered width of `staff`
    width: f32,
    pub staff_interaction: StaffInteractionState,
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
            staff_interaction: StaffInteractionState::default(),
        }
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_font(&self) -> iced::Font {
        self.font.font_iced
    }

    pub fn get_bars(&self) -> &Vec<BarEl> {
        &self.bars
    }

    pub fn get_midi_messages(&self) -> Vec<MidiMessageTimed> {
        let mut ret = vec![];
        for bar in self.get_bars() {
            for note in bar.get_notes() {
                let mut midi_messages = match note.get_pitch() {
                    Some(pitch) => vec![
                        MidiMessageTimed::new(
                            MidiMessage::note_on(0, pitch.to_midi_note_number(), 100),
                            note.get_time_duration(),
                        ),
                        MidiMessageTimed::new(
                            MidiMessage::note_off_channel(0, false),
                            Duration::ZERO,
                        ),
                    ],
                    None => vec![MidiMessageTimed::new(
                        MidiMessage::note_off_channel(0, false),
                        note.get_time_duration(),
                    )],
                };

                ret.append(&mut midi_messages);
            }
        }

        ret
    }

    pub fn add_bar(self: &mut Self) {
        let new_bar = Bar::new(vec![NoteOrRest::new(None, note_or_rest::BaseDuration(1))]);
        let new_el = BarEl::new(new_bar, self.width, &self.font);
        self.width += new_el.get_width();
        self.bars.push(new_el);
    }

    /// Setting to None changes the note to a rest
    ///
    /// May change the layout of self
    pub fn set_note_pitch(self: &mut Self, staff_index: &StaffIndex, pitch: Option<Pitch>) {
        let bar = &mut self.bars[staff_index.bar_index];
        let w = bar.get_width();
        bar.set_note_pitch(staff_index.note_index, pitch, &self.font);
        let dw = bar.get_width() - w;
        self.fix_layout(staff_index.bar_index, dw);
    }

    /// May change the layout of self
    pub fn set_note_base_duration(
        self: &mut Self,
        staff_index: &StaffIndex,
        base_duration: BaseDuration,
    ) {
        let bar = &mut self.bars[staff_index.bar_index];
        let w = bar.get_width();
        bar.set_note_base_duration(staff_index.note_index, base_duration, &self.font);
        let dw = bar.get_width() - w;
        self.fix_layout(staff_index.bar_index, dw);
    }

    pub fn redraw(&mut self) {
        self.cache.clear();
    }

    /// Fix the layout after modifying bar at `bar_index` by width `d_width`
    fn fix_layout(&mut self, bar_index: usize, d_width: f32) {
        for i in bar_index + 1..self.bars.len() {
            self.bars[i].translate_x(d_width);
        }
        self.width += d_width;
    }
}

#[derive(Default)]
pub struct State {
    hovering_new_bar: bool,
    new_bar_button_bounds: Rectangle,
    staff_transformation: StaffTransformation,
}

/// The transformation applied to the staff, ie. transformation from widget to staff space
///
/// To transform the staff, scale is applied first, and then translation
struct StaffTransformation {
    scale: f32,
    translation: Vector,
}

impl Default for StaffTransformation {
    fn default() -> Self {
        Self {
            scale: 0.1,
            translation: Vector::new(0., 8. * STANDARD_STAFF_SPACING),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum StaffInteractionState {
    /// Not dragging a note, could be hovering/selecting a note or both
    NotDragging {
        hovering: Option<StaffIndex>,
        selected: Option<StaffIndex>,
    },
    /// Dragging note at `StaffIndex` with cursor
    Dragging(StaffIndex),
}

impl Default for StaffInteractionState {
    fn default() -> Self {
        Self::NotDragging {
            hovering: None,
            selected: None,
        }
    }
}

impl StaffInteractionState {
    /// Returns the index of the selected note, if one is selected
    pub fn get_selected(&self) -> Option<&StaffIndex> {
        match self {
            Self::NotDragging { selected, .. } => selected.as_ref(),
            _ => None,
        }
    }

    pub const NONE: Self = Self::NotDragging {
        hovering: None,
        selected: None,
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StaffIndex {
    /// Which bar the note is in
    bar_index: usize,
    /// The index of the note within its bar
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

fn draw_bar_lines(frame: &mut Frame, bounds: Rectangle, thickness: f32) {
    let spacing = bounds.height / 4.;
    for i in 0..5 {
        let y = i as f32 * spacing;
        let from = Point::new(bounds.x, y - thickness / 2.);
        let path = canvas::Path::rectangle(from, iced::Size::new(bounds.width, thickness));
        frame.fill(&path, Color::BLACK);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StaffInteractionMsg {
    /// Start hovering the note at `StaffIndex`
    StartHovering(StaffIndex),
    /// Stop hovering note
    StopHovering,
    /// Select note at `StaffIndex`
    SelectNote(StaffIndex),
    /// Unselect the currently selected note
    UnselectNote,
    /// Start dragging the note at `StaffIndex`
    StartDragging(StaffIndex),
    /// Drag the currently dragged note to `Pitch`
    DragToPitch(Pitch),
    /// Stop dragging the note
    StopDragging,
}

pub fn handle_staff_interaction_msg(
    staff_interaction_msg: StaffInteractionMsg,
    staff: &mut StaffEl,
) {
    // Whether or not to trigger a redraw
    let mut staff_modified = false;

    // Triggers a redraw if the staff_interaction changed
    macro_rules! update_staff_interaction {
        ($new_staff_interaction:expr) => {
            let new_staff_interaction = $new_staff_interaction;
            if new_staff_interaction != staff.staff_interaction {
                staff.staff_interaction = new_staff_interaction;
                staff_modified = true;
            }
        };
    }

    match staff_interaction_msg {
        StaffInteractionMsg::StartHovering(staff_index) => {
            if let StaffInteractionState::NotDragging { selected, .. } = staff.staff_interaction {
                update_staff_interaction!(StaffInteractionState::NotDragging {
                    hovering: Some(staff_index),
                    selected,
                });
            }
        }
        StaffInteractionMsg::StopHovering => {
            if let StaffInteractionState::NotDragging { selected, .. } = staff.staff_interaction {
                update_staff_interaction!(StaffInteractionState::NotDragging {
                    hovering: None,
                    selected,
                });
            }
        }
        StaffInteractionMsg::StartDragging(staff_index) => {
            update_staff_interaction!(StaffInteractionState::Dragging(staff_index));
        }
        StaffInteractionMsg::DragToPitch(pitch) => {
            if let StaffInteractionState::Dragging(staff_index) = &staff.staff_interaction {
                staff.set_note_pitch(&staff_index.clone(), Some(pitch));
                staff_modified = true;
            }
        }
        StaffInteractionMsg::StopDragging => {
            if let StaffInteractionState::Dragging(staff_index) = &staff.staff_interaction {
                update_staff_interaction!(StaffInteractionState::NotDragging {
                    selected: Some(staff_index.clone()),
                    hovering: None,
                });
            }
        }
        StaffInteractionMsg::SelectNote(staff_index) => {
            if let StaffInteractionState::NotDragging { hovering, .. } = staff.staff_interaction {
                update_staff_interaction!(StaffInteractionState::NotDragging {
                    hovering,
                    selected: Some(staff_index),
                });
            }
        }
        StaffInteractionMsg::UnselectNote => {
            if let StaffInteractionState::NotDragging { hovering, .. } = staff.staff_interaction {
                update_staff_interaction!(StaffInteractionState::NotDragging {
                    hovering,
                    selected: None,
                });
            }
        }
    }

    if staff_modified {
        staff.redraw();
    }
}

impl canvas::Program<ScoreEditingMessage> for StaffEl {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<ScoreEditingMessage>> {
        // TODO: Clean up update logic
        let new_bar_width = 4. * STANDARD_STAFF_SPACING;
        let new_bar_button_bounds = Rectangle {
            x: self.get_width(),
            y: 0.,
            width: new_bar_width,
            height: 4. * STANDARD_STAFF_SPACING,
        };
        state.new_bar_button_bounds = new_bar_button_bounds;
        let cursor_position = cursor
            .position_in(bounds)
            .map(|p| widget_to_staff_space(p, &state.staff_transformation));

        match event {
            iced::Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(button) => {
                    match button {
                        mouse::Button::Left => {
                            if let Some(position) = cursor_position {
                                if state.new_bar_button_bounds.contains(position) {
                                    return Some(canvas::Action::publish(
                                        ScoreEditingMessage::AddBar,
                                    ));
                                }

                                match self.staff_interaction {
                                    StaffInteractionState::NotDragging { hovering, selected } => {
                                        if let Some(staff_index) = hovering {
                                            return Some(canvas::Action::publish(
                                                ScoreEditingMessage::StaffInteractionMsg(
                                                    StaffInteractionMsg::StartDragging(staff_index),
                                                ),
                                            ));
                                        }

                                        if let Some(staff_index) = selected {
                                            if get_hovering(&position, self) != Some(staff_index) {
                                                return Some(canvas::Action::publish(
                                                    ScoreEditingMessage::StaffInteractionMsg(
                                                        StaffInteractionMsg::UnselectNote,
                                                    ),
                                                ));
                                            }
                                        }
                                    }
                                    StaffInteractionState::Dragging(..) => {}
                                }
                            }
                        }
                        _ => return None,
                    }
                    return None;
                }
                mouse::Event::CursorMoved { .. } => {
                    match cursor_position {
                        Some(position) => {
                            let new_bar_state_changed;
                            if state.new_bar_button_bounds.contains(position) {
                                new_bar_state_changed = !state.hovering_new_bar;
                                state.hovering_new_bar = true;
                            } else {
                                new_bar_state_changed = state.hovering_new_bar;
                                state.hovering_new_bar = false;
                            }
                            if new_bar_state_changed {
                                self.cache.clear();
                                return Some(Action::request_redraw());
                            }

                            let hovering = get_hovering(&position, self);
                            match self.staff_interaction {
                                StaffInteractionState::NotDragging { .. } => match hovering {
                                    Some(hovering_index) => {
                                        return Some(canvas::Action::publish(
                                            ScoreEditingMessage::StaffInteractionMsg(
                                                StaffInteractionMsg::StartHovering(hovering_index),
                                            ),
                                        ));
                                    }
                                    None => {
                                        return Some(canvas::Action::publish(
                                            ScoreEditingMessage::StaffInteractionMsg(
                                                StaffInteractionMsg::StopHovering,
                                            ),
                                        ));
                                    }
                                },
                                StaffInteractionState::Dragging(..) => {
                                    if let Some(pitch) = Pitch::from_y_offset(position.y) {
                                        return Some(canvas::Action::publish(
                                            ScoreEditingMessage::StaffInteractionMsg(
                                                StaffInteractionMsg::DragToPitch(pitch),
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                        None => {
                            if state.hovering_new_bar {
                                state.hovering_new_bar = false;
                                self.cache.clear();
                                return Some(Action::request_redraw());
                            }
                        }
                    }
                    return None;
                }
                mouse::Event::ButtonReleased(button) => match button {
                    mouse::Button::Left => match self.staff_interaction {
                        StaffInteractionState::NotDragging { .. } => None,
                        StaffInteractionState::Dragging(..) => {
                            Some(Action::publish(ScoreEditingMessage::StaffInteractionMsg(
                                StaffInteractionMsg::StopDragging,
                            )))
                        }
                    },
                    _ => None,
                },
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
                f.scale(state.staff_transformation.scale);
                f.translate(state.staff_transformation.translation);

                let staff_bar_line_bounds = Rectangle {
                    x: 0.,
                    y: 0.,
                    width: self.get_width(),
                    height: 400.,
                };
                draw_bar_lines(
                    f,
                    staff_bar_line_bounds,
                    self.font.engraving_defaults.staff_line_thickness,
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
                    let bar_interaction = match self.staff_interaction {
                        StaffInteractionState::NotDragging { hovering, selected } => {
                            BarInteraction::NotDragging {
                                hovering: hovering.and_then(|h| {
                                    if h.bar_index == i {
                                        Some(h.note_index)
                                    } else {
                                        None
                                    }
                                }),
                                selected: selected.and_then(|h| {
                                    if h.bar_index == i {
                                        Some(h.note_index)
                                    } else {
                                        None
                                    }
                                }),
                            }
                        }
                        StaffInteractionState::Dragging(dragging) => {
                            if dragging.bar_index == i {
                                BarInteraction::Dragging(dragging.note_index)
                            } else {
                                BarInteraction::NONE
                            }
                        }
                    };
                    bar.draw(f, &bar_interaction, &self.font);
                }
            });
        });
        vec![geom]
    }
}

/// Widget space is the space within the bounds of the canvas widget, with no scale applied
fn widget_to_staff_space(point: Point, staff_transformation: &StaffTransformation) -> Point {
    Point::new(
        point.x / staff_transformation.scale,
        point.y / staff_transformation.scale,
    ) - staff_transformation.translation
}
