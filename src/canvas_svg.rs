use iced::{
    Point, Rectangle, Size, Vector,
    widget::{canvas::Frame, svg::Handle},
};

pub enum Positioning {
    Centered(Point<f32>),
    TopLeft(Point<f32>),
}

#[allow(dead_code)]
pub enum SizingMode {
    WidthOnly(f32),
    HeightOnly(f32),
    Fixed(Size<f32>),
}

#[allow(dead_code)]
pub struct CanvasSVG {
    path: &'static str,
    // `aspect_ratio` is Width / Height
    aspect_ratio: f32,
    size: Size<f32>,
    top_left: Point<f32>,
}

impl CanvasSVG {
    pub fn new(
        path: &'static str,
        aspect_ratio: f32,
        position: Positioning,
        size: SizingMode,
    ) -> Self {
        let size = match size {
            SizingMode::WidthOnly(width) => Size::new(width, width / aspect_ratio),
            SizingMode::HeightOnly(height) => Size::new(height * aspect_ratio, height),
            SizingMode::Fixed(size) => size,
        };

        let top_left = match position {
            Positioning::TopLeft(pos) => pos,
            Positioning::Centered(pos) => pos - Vector::from(size / 2.0),
        };

        Self {
            path: path,
            aspect_ratio: aspect_ratio,
            size: size,
            top_left: top_left,
        }
    }

    pub fn draw_to_frame(self, frame: &mut Frame) {
        let handle = Handle::from_path(self.path);
        let bounds = Rectangle::new(self.top_left, self.size);
        frame.draw_svg(bounds, &handle);
    }
}
