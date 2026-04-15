use std::collections::HashMap;

use iced::{Renderer, Theme, mouse, widget::canvas};

use crate::{
    Message, Pitch,
    canvas_el::CanvasEl,
    staff::{self, PitchClass, StaffState, StaffWidget},
};

pub struct Score {
    staffs: HashMap<u32, Staff>,
}

pub struct Staff {
    pub notes: Vec<Pitch>,
    pub y: f32,
}

#[derive(Default)]
pub struct ScoreState<'a> {
    staff_els: HashMap<u32, CanvasEl<StaffWidget<'a>>>,
}

impl Default for Score {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            0,
            Staff {
                notes: vec![(PitchClass::C, 4)],
                y: 0.,
            },
        );
        map.insert(
            1,
            Staff {
                notes: vec![(PitchClass::E, 4)],
                y: 100.,
            },
        );

        Self { staffs: map }
    }
}

impl canvas::Program<Message> for Score {
    type State = ScoreState<'static>;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        state
            .staff_els
            .iter()
            .filter_map(|(key, staff_el)| {
                let staff = self.staffs.get(key)?;
                let props = staff::Props {
                    notes: &staff.notes,
                };

                Some(staff_el.draw(props, renderer, &bounds))
            })
            .collect()
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let staff_els = &mut state.staff_els;

        for (key, staff) in &self.staffs {
            let staff_el = staff_els
                .entry(*key)
                .or_insert(CanvasEl::<StaffState>::default());

            let props = staff::Props {
                notes: &staff.notes,
            };

            match staff_el.update(event, props, bounds, cursor) {
                Some(update) => {
                    return Some(update.map_message(|message| match message {
                        // staff::Message::AddNote(note) => Message::AddNote(note, *key),
                    }));
                }
                None => (),
            };
        }

        None
    }
}
