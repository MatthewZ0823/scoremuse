use core::f32;

use crate::canvas_svg::{CanvasSVG, Positioning::*, SizingMode::*};
use iced::widget::Action;
use iced::widget::canvas::{self};
use iced::{Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::Message;

#[derive(Debug, Default)]
pub struct Staff {
    cache: canvas::Cache,
}

const TREBLE_CLEF_ASPECT_RATIO: f32 = 95.116 / 153.12;
const TREBLE_CLEF_PATH: &str = "src/assets/treble_clef.svg";

const FILLED_NOTE_HEAD_ASPECT_RATIO: f32 = 260. / 200.;
const FILLED_NOTE_HEAD_PATH: &str = "src/assets/filled_note_head.svg";

impl canvas::Program<Message> for Staff {
    type State = bool;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let iced::Event::Mouse(event) = event {
            if let mouse::Event::CursorMoved { position } = event {
                *state = position.y > 60.;
                self.cache.clear();
                Some(Action::request_redraw().and_capture())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            let width = bounds.size().width;
            let _height = bounds.size().height;

            println!("Redrawing...");

            let staff_spacing = 20.;

            for i in 1..6 {
                let y = i as f32 * staff_spacing;
                let from = Point::new(10.0, y);
                let to = Point::new(width - 10.0, y);
                let line = canvas::Path::line(from, to);

                let color = if *state {
                    iced::Color::from_rgb(0., 1., 0.)
                } else {
                    iced::Color::from_rgb(1., 0., 0.)
                };

                frame.stroke(&line, canvas::Stroke::default().with_color(color));
            }

            let treble_clef = CanvasSVG::new(
                TREBLE_CLEF_PATH,
                TREBLE_CLEF_ASPECT_RATIO,
                TopLeft(Point::new(0., -0.25 * staff_spacing)),
                HeightOnly(7. * staff_spacing),
            );
            treble_clef.draw_to_frame(frame);

            let note = CanvasSVG::new(
                FILLED_NOTE_HEAD_PATH,
                FILLED_NOTE_HEAD_ASPECT_RATIO,
                Centered(Point::new(100., 100.)),
                HeightOnly(20.),
            );
            note.draw_to_frame(frame);
        });

        vec![geom]
    }
}
