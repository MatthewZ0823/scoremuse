use crate::constants::STANDARD_STAFF_SPACING;

/// Draws `glyph` on the given `frame`, positioned to render SMuFL glyphs nicely on the score.
///
/// - `position` is where the origin of the glyph is placed on the frame.
/// - `color`, if `None`, falls back to the glyph's default color.
///
/// Returns the color the glyph was actually drawn with.
pub fn draw_smufl_glyph(
    frame: &mut iced::widget::canvas::Frame,
    glyph: char,
    position: iced::Point,
    color: Option<iced::Color>,
    font: &iced::font::Font,
) -> iced::Color {
    let glyph: iced::widget::canvas::Text = iced::widget::canvas::Text {
        font: *font,
        align_y: iced::alignment::Vertical::Center,
        position: position,
        size: (4. * STANDARD_STAFF_SPACING).into(),
        ..Into::<String>::into(glyph).into()
    };

    let mut _color: iced::Color = Default::default();
    glyph.draw_with(|path, c| {
        _color = color.unwrap_or(c);
        frame.fill(&path, color.unwrap_or(c));
    });
    _color
}
