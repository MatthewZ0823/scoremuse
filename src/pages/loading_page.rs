use iced::{Element, widget::text};

use crate::font::{self, FontMeta};

#[derive(Default)]
pub struct LoadingPage {}

impl LoadingPage {
    pub fn view(&self) -> Element<'_, LoadingMessage> {
        text("loading font...").into()
    }

    /// Returns the font meta data once it's loaded
    pub fn update(&mut self, loading_message: LoadingMessage) -> FontMeta {
        match loading_message {
            LoadingMessage::FontLoaded(res) => {
                let font_meta = res.unwrap(); // TODO: Better error handling
                font_meta
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadingMessage {
    FontLoaded(Result<FontMeta, font::Error>),
}
