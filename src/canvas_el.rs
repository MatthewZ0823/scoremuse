use iced::{
    Rectangle, Renderer, Size, Transformation, mouse,
    widget::canvas::{self, Cache, Geometry},
};

pub enum CWidgetUpdate<M> {
    MESSAGE(M),
    REDRAW,
}

pub trait CanvasWidget {
    type State;
    type Message;

    fn draw_fn(&self, state: &Self::State, size: &Size, frame: &mut canvas::Frame<Renderer>);
    fn update_fn(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<CWidgetUpdate<Self::Message>>;
}

pub struct TransformableWidget<W: CanvasWidget> {
    transformation: Transformation,
    widget: W,
}

impl<W: CanvasWidget> CanvasWidget for TransformableWidget<W> {
    type State = W::State;
    type Message = W::Message;

    fn draw_fn(&self, state: &Self::State, size: &Size, frame: &mut canvas::Frame<Renderer>) {
        frame.translate(self.transformation.translation());
        frame.scale(self.transformation.scale_factor());
        self.widget.draw_fn(state, size, frame);
    }

    fn update_fn(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<CWidgetUpdate<Self::Message>> {
        self.widget.update_fn(state, event, bounds, cursor)
    }
}

impl<W: CanvasWidget> TransformableWidget<W> {
    /// Left multiply the current transformation by `transform`
    /// has the affect of applying `transform` after the current transformations
    pub fn update_transformation(&mut self, transform: Transformation) {
        self.transformation = transform * self.transformation;
    }

    /// Apply a translation this `CanvasEl`
    pub fn translate(&mut self, x: f32, y: f32) {
        self.update_transformation(Transformation::translate(x, y));
    }
}

#[derive(Default)]
pub struct CanvasEl<W: CanvasWidget> {
    cache: canvas::Cache,
    widget: W,
}

impl<W> CanvasEl<W>
where
    W: CanvasWidget,
{
    pub fn new(widget: W) -> Self {
        Self {
            cache: Cache::new(),
            widget,
        }
    }

    pub fn draw(
        &self,
        state: &W::State,
        renderer: &Renderer,
        bounds: &Rectangle,
    ) -> Geometry<Renderer> {
        self.cache.draw(renderer, bounds.size(), |frame| {
            self.widget.draw_fn(state, &bounds.size(), frame);
        })
    }

    pub fn update(
        &self,
        state: &mut W::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<W::Message>> {
        let update_message = self.widget.update_fn(state, event, bounds, cursor);

        update_message.map(|msg| match msg {
            CWidgetUpdate::REDRAW => {
                self.cache.clear();
                canvas::Action::request_redraw()
            }
            CWidgetUpdate::MESSAGE(m) => canvas::Action::publish(m),
        })
    }
}
