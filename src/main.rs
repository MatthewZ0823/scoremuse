use crate::staff::Staff;
use iced::Fill;
use iced::widget::canvas;
use iced::widget::{button, column, text};
use iced::{Color, Rectangle, Renderer, Theme, Vector};
use iced::{Element, mouse};

mod staff;
mod canvas_svg;

const DEBUG: bool = false;

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug)]
struct Circle {
    radius: f32,
}

// Then, we implement the `Program` trait
impl<Message> canvas::Program<Message> for Circle {
    // No internal state
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let mut frame2 = canvas::Frame::new(renderer, bounds.size());

        let offset = Vector::new(10.0, 10.0);

        // We create a `Path` representing a simple circle
        let circle = canvas::Path::circle(frame.center(), self.radius);
        let circle2 = canvas::Path::circle(frame2.center() + offset, self.radius);

        // And fill it with some color
        frame.fill(&circle, Color::from_rgb(1.0, 0.0, 0.0));
        frame2.fill(&circle2, Color::from_rgb(0.0, 1.0, 0.0));

        // Then, we produce the geometry
        vec![frame.into_geometry(), frame2.into_geometry()]
    }
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The buttons
        let increment = button("+").on_press(Message::Increment);
        let decrement = button("-").on_press(Message::Decrement);

        // The number
        let counter = text(self.value).size(100);

        let staff = canvas(Staff::default()).width(Fill).height(Fill);

        // The layout
        let interface: Element<_> = column![increment, counter, decrement, staff]
            .height(Fill)
            .width(Fill)
            .into();

        let explained = if DEBUG {
            interface.explain(Color::BLACK)
        } else {
            interface
        };

        explained
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[test]
fn it_counts_properly() {
    let mut counter = Counter { value: 0 };

    counter.update(Message::Increment);
    counter.update(Message::Increment);
    counter.update(Message::Decrement);

    assert_eq!(counter.value, 1);
}
