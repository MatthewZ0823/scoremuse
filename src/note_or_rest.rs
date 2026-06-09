use std::ops::{Div, Mul};

use iced::Size;
use iced::widget::canvas::path::lyon_path::geom::euclid::{Transform2D, Vector2D};
use iced::widget::canvas::{self, Style};
use iced::{
    Color, Point, Radians, Vector,
    widget::canvas::fill::Rule,
    widget::canvas::{Fill, Frame, Path, path::arc::Elliptical},
};

use crate::colors::HIGHLIGHT_COLOR;
use crate::constants::BARLINE_Y_SPACING;
use crate::pitch::{Pitch, PitchClass};
use crate::staff::pitch_to_y_offset;
use crate::utils::style_from_color;

#[derive(Debug)]
pub struct NoteOrRest {
    // When pitch is None its a rest
    pitch: Option<Pitch>,
    // 1 -> Whole Note, 2 -> Half Note, 3 - Quarter Note, ...
    duration: u8,
}

impl NoteOrRest {
    pub fn new(pitch: Option<Pitch>, duration: u8) -> Self {
        NoteOrRest {
            pitch: pitch,
            duration: duration,
        }
    }
}

pub struct NoteOrRestEl {
    note_or_rest: NoteOrRest,
    // X-Coordinate of the note head/rest center
    x: f32,
    // How much space is on the left of the note's center
    // TODO: Don't like how this depends on whether its the first note or not
    left_width: f32,
    // How much space to put after the note
    right_width: f32,
}

pub enum NoteInteraction {
    None,
    // Hovering this note
    Hovering,
    // Selected this note and mouse is hovering pitch
    Selected(Pitch),
}

impl NoteOrRestEl {
    // x is the left-most x coordinate of the note
    // `is_first` iff the note is the first note of its bar
    pub fn new(note_or_rest: NoteOrRest, x: f32, is_first: bool) -> Self {
        let (left_width, right_width) = Self::compute_widths(&note_or_rest, is_first);
        Self {
            note_or_rest,
            x: x + left_width,
            left_width,
            right_width,
        }
    }

    // Returns (left_width, right_width)
    fn compute_widths(note_or_rest: &NoteOrRest, is_first: bool) -> (f32, f32) {
        (
            (if is_first { 2. } else { 0.75 }) * BARLINE_Y_SPACING,
            match note_or_rest.duration {
                1 => 6.25,
                2 => 3.25,
                3 => 2.25,
                _ => {
                    if note_or_rest.pitch.map_or(false, |p| Self::stem_down(&p)) {
                        1.25
                    } else {
                        2.25
                    }
                }
            } * BARLINE_Y_SPACING,
        )
    }

    fn stem_down(pitch: &Pitch) -> bool {
        *pitch > Pitch::new(PitchClass::B, 4)
    }

    /// Gets the x-coordinate of the right bound
    pub fn get_right_bound(self: &Self) -> f32 {
        self.x + self.right_width
    }

    /// Gets the x-coordinate of the left bound
    pub fn get_left_bound(self: &Self) -> f32 {
        self.x - self.left_width
    }

    // Get the total width of the note
    pub fn get_width(self: &Self) -> f32 {
        self.left_width + self.right_width
    }

    pub fn translate_x(self: &mut Self, dx: f32) {
        self.x += dx;
    }

    // Keeps the center the same
    // Assuming only the right width might change on pitch change
    pub fn set_pitch(self: &mut Self, pitch: Pitch) {
        self.note_or_rest.pitch = Some(pitch);
        (_, self.right_width) = Self::compute_widths(&self.note_or_rest, false);
    }

    pub fn draw(self: &Self, frame: &mut Frame, note_interaction: &NoteInteraction) {
        let style = match note_interaction {
            NoteInteraction::None => style_from_color(Color::BLACK),
            NoteInteraction::Hovering | NoteInteraction::Selected(..) => {
                style_from_color(HIGHLIGHT_COLOR)
            }
        };

        // Draw the "selected/hover" note
        if let NoteInteraction::Selected(hovering_pitch) = note_interaction {
            let stem_direction = if Self::stem_down(hovering_pitch) {
                StemDirection::DOWN
            } else {
                StemDirection::UP
            };

            draw_note(
                frame,
                self.note_or_rest.duration,
                Point::new(self.x, pitch_to_y_offset(hovering_pitch)),
                BARLINE_Y_SPACING,
                stem_direction,
                style_from_color(Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        match &self.note_or_rest.pitch {
            None => match self.note_or_rest.duration {
                1 => {
                    draw_rect_rest(
                        frame,
                        BARLINE_Y_SPACING / 2.,
                        Point::new(self.x, BARLINE_Y_SPACING * 1.25),
                        style,
                    );
                }
                2 => {
                    draw_rect_rest(
                        frame,
                        BARLINE_Y_SPACING / 2.,
                        Point::new(self.x, BARLINE_Y_SPACING * 1.75),
                        style,
                    );
                }
                3 => {
                    draw_quarter_rest(
                        frame,
                        BARLINE_Y_SPACING * 2.5,
                        Point::new(self.x, BARLINE_Y_SPACING * 2.),
                        style,
                    );
                }
                _ => todo!(),
            },
            Some(pitch) => {
                let center = Point::new(self.x, pitch_to_y_offset(&pitch));
                let stem_direction = if Self::stem_down(pitch) {
                    StemDirection::DOWN
                } else {
                    StemDirection::UP
                };
                draw_note(
                    frame,
                    self.note_or_rest.duration,
                    center,
                    BARLINE_Y_SPACING,
                    stem_direction,
                    style,
                );
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum StemDirection {
    UP,
    DOWN,
}

/// Draws a note to `frame`, where the note head is centered at `center` and has height `height`
fn draw_note(
    frame: &mut Frame,
    duration: u8,
    center: Point,
    height: f32,
    stem_direction: StemDirection,
    style: Style,
) {
    match duration {
        1 => draw_whole_note_head(frame, height, center, style),
        2 => {
            draw_half_note_head(frame, height, center, style.clone());
            draw_stem(frame, height, center, stem_direction, 0, style);
        }
        d => {
            draw_filled_note_head(frame, height, center, style.clone());
            draw_stem(frame, height, center, stem_direction, d - 3, style);
        }
    }
}

fn draw_filled_note_head(frame: &mut Frame, height: f32, center: Point, style: Style) {
    // Half-height of ellipse rotated at 15 degrees with radii 2,3
    const MAGIC: f32 = 2.0820510297634165775;

    let filled_head_path: Path = Path::new(|b| {
        b.ellipse(Elliptical {
            center: center,
            radii: Vector::new(1.5 / MAGIC * height, 1. / MAGIC * height),
            rotation: Radians::PI.div(-12.),
            start_angle: 0.into(),
            end_angle: Radians::PI.mul(2.),
        });
    });

    let fill = Fill {
        style: style.into(),
        rule: Rule::NonZero,
    };
    frame.fill(&filled_head_path, fill);
}

fn draw_half_note_head(frame: &mut Frame, height: f32, center: Point, style: Style) {
    // Half-height of ellipse rotated at 15 degrees with radii 2,3
    const MAGIC: f32 = 2.0820510297634165775;

    let half_head_path: Path = Path::new(|b| {
        b.ellipse(Elliptical {
            center: center,
            radii: Vector::new(1.5 / MAGIC * height, 1. / MAGIC * height),
            rotation: Radians::PI.div(-12.),
            start_angle: 0.into(),
            end_angle: Radians::PI.mul(2.),
        });

        b.ellipse(Elliptical {
            center: center,
            radii: Vector::new(1.3 / MAGIC * height, 0.5 / MAGIC * height),
            rotation: Radians::PI.div(-12.),
            start_angle: 0.into(),
            end_angle: Radians::PI.mul(2.),
        });
    });

    frame.fill(
        &half_head_path,
        Fill {
            style: style.into(),
            rule: Rule::EvenOdd,
        },
    );
}

fn draw_whole_note_head(frame: &mut Frame, mut height: f32, center: Point, style: Style) {
    height /= 2.;

    let whole_head_path: Path = Path::new(|b| {
        b.ellipse(Elliptical {
            center: center,
            radii: Vector::new(1.5 * height, height),
            rotation: 0.into(),
            start_angle: 0.into(),
            end_angle: Radians::PI.mul(2.),
        });

        b.ellipse(Elliptical {
            center: center,
            radii: Vector::new(0.6 * height, 0.9 * height),
            rotation: Radians::PI.mul(-0.25),
            start_angle: 0.into(),
            end_angle: Radians::PI.mul(2.),
        });
    });

    frame.fill(
        &whole_head_path,
        Fill {
            style: style.into(),
            rule: Rule::EvenOdd,
        },
    );
}

fn draw_quarter_rest(frame: &mut Frame, mut height: f32, center: Point, style: Style) {
    height /= 2.;
    let transform =
        Transform2D::scale(height, height).then_translate(Vector2D::new(center.x, center.y));
    let quarter_rest_path: Path = Path::new(|b| {
        b.move_to(Point::new(-0.07, -0.98));
        b.bezier_curve_to(
            Point::new(-0.0892, -0.9719),
            Point::new(-0.1007, -0.9439),
            Point::new(-0.0918, -0.9243),
        );
        b.bezier_curve_to(
            Point::new(-0.0892, -0.9215),
            Point::new(-0.0613, -0.8882),
            Point::new(-0.0333, -0.8515),
        );
        b.bezier_curve_to(
            Point::new(0.0307, -0.7794),
            Point::new(0.0416, -0.7623),
            Point::new(0.0557, -0.729),
        );
        b.bezier_curve_to(
            Point::new(0.1116, -0.6147),
            Point::new(0.0809, -0.4693),
            Point::new(-0.0169, -0.3773),
        );
        b.bezier_curve_to(
            Point::new(-0.0252, -0.3664),
            Point::new(-0.0613, -0.3359),
            Point::new(-0.0952, -0.3107),
        );
        b.bezier_curve_to(
            Point::new(-0.1925, -0.2269),
            Point::new(-0.2373, -0.1793),
            Point::new(-0.2538, -0.1373),
        );
        b.bezier_curve_to(
            Point::new(-0.2598, -0.1263),
            Point::new(-0.2598, -0.1154),
            Point::new(-0.2598, -0.0984),
        );
        b.bezier_curve_to(
            Point::new(-0.2625, -0.0596),
            Point::new(-0.2598, -0.0563),
            Point::new(-0.1449, 0.0771),
        );
        b.bezier_curve_to(
            Point::new(0.0109, 0.2642),
            Point::new(0.1225, 0.3954),
            Point::new(0.1312, 0.4036),
        );
        b.bezier_curve_to(
            Point::new(-0.0252, 0.3423),
            Point::new(-0.198, 0.3116),
            Point::new(-0.2565, 0.3396),
        );
        b.bezier_curve_to(
            Point::new(-0.2762, 0.3478),
            Point::new(-0.2877, 0.3592),
            Point::new(-0.2958, 0.3784),
        );
        b.bezier_curve_to(
            Point::new(-0.3184, 0.426),
            Point::new(-0.3123, 0.496),
            Point::new(-0.2789, 0.5988),
        );
        b.bezier_curve_to(
            Point::new(-0.2484, 0.6912),
            Point::new(-0.187, 0.8137),
            Point::new(-0.1259, 0.9057),
        );
        b.bezier_curve_to(
            Point::new(-0.1007, 0.945),
            Point::new(-0.0531, 1.0062),
            Point::new(-0.0476, 1.009),
        );
        b.bezier_curve_to(
            Point::new(-0.0393, 1.0172),
            Point::new(-0.0279, 1.0144),
            Point::new(-0.0197, 1.009),
        );
        b.bezier_curve_to(
            Point::new(-0.0115, 0.9981),
            Point::new(-0.0115, 0.9892),
            Point::new(-0.0279, 0.9702),
        );
        b.bezier_curve_to(
            Point::new(-0.0864, 0.8865),
            Point::new(-0.1142, 0.7132),
            Point::new(-0.0809, 0.6212),
        );
        b.bezier_curve_to(
            Point::new(-0.0673, 0.5797),
            Point::new(-0.0503, 0.5572),
            Point::new(-0.0197, 0.5431),
        );
        b.bezier_curve_to(
            Point::new(0.0612, 0.5069),
            Point::new(0.2401, 0.5517),
            Point::new(0.315, 0.6266),
        );
        b.bezier_curve_to(
            Point::new(0.3205, 0.6322),
            Point::new(0.3319, 0.6437),
            Point::new(0.3374, 0.6464),
        );
        b.bezier_curve_to(
            Point::new(0.3571, 0.6546),
            Point::new(0.385, 0.6437),
            Point::new(0.3933, 0.624),
        );
        b.bezier_curve_to(
            Point::new(0.4047, 0.6042),
            Point::new(0.3987, 0.5907),
            Point::new(0.3735, 0.56),
        );
        b.bezier_curve_to(
            Point::new(0.3265, 0.5041),
            Point::new(0.1842, 0.3368),
            Point::new(0.1646, 0.3116),
        );
        b.bezier_curve_to(
            Point::new(0.1142, 0.2531),
            Point::new(0.0918, 0.1974),
            Point::new(0.0864, 0.1274),
        );
        b.bezier_curve_to(
            Point::new(0.0837, 0.0382),
            Point::new(0.1197, -0.0563),
            Point::new(0.187, -0.1182),
        );
        b.bezier_curve_to(
            Point::new(0.1952, -0.1291),
            Point::new(0.2313, -0.1597),
            Point::new(0.2647, -0.1848),
        );
        b.bezier_curve_to(
            Point::new(0.3681, -0.2713),
            Point::new(0.4102, -0.3188),
            Point::new(0.4266, -0.3637),
        );
        b.bezier_curve_to(
            Point::new(0.4381, -0.3998),
            Point::new(0.4326, -0.4332),
            Point::new(0.4074, -0.4638),
        );
        b.bezier_curve_to(
            Point::new(0.3987, -0.4719),
            Point::new(0.3013, -0.5923),
            Point::new(0.187, -0.729),
        );
        b.bezier_curve_to(
            Point::new(0.0307, -0.9128),
            Point::new(-0.0252, -0.98),
            Point::new(-0.0333, -0.9828),
        );
        b.bezier_curve_to(
            Point::new(-0.0448, -0.9855),
            Point::new(-0.0585, -0.9855),
            Point::new(-0.07, -0.98),
        );
        b.close();
    })
    .transform(&transform);

    let fill = Fill {
        style: style.into(),
        rule: Rule::NonZero,
    };
    frame.fill(&quarter_rest_path, fill);
}

fn draw_rect_rest(frame: &mut Frame, height: f32, center: Point, style: Style) {
    let rect_rest_path: Path = Path::new(|b| {
        b.rectangle(
            Point::new(center.x - height * 1.25, center.y - height / 2.),
            Size::new(2.5 * height, height),
        );
    });
    let fill = Fill {
        style: style.into(),
        rule: Rule::NonZero,
    };
    frame.fill(&rect_rest_path, fill);
}

// `height` and `center` are the height and center of the note that the stem is drawn on
fn draw_stem(
    frame: &mut Frame,
    height: f32,
    center: Point,
    stem_direction: StemDirection,
    num_tails: u8,
    style: Style,
) {
    let sign = match stem_direction {
        StemDirection::UP => 1.,
        StemDirection::DOWN => -1.,
    };
    let (stem_start, stem_end) = {
        let stem_x = center.x + sign * 0.65 * height;
        // Added length from tails
        let added_length = (num_tails as f32 - 1.).max(0.) * height;
        (
            Point::new(stem_x, center.y),
            Point::new(stem_x, center.y - sign * (3.5 * height + added_length)),
        )
    };
    let stem_path = Path::line(stem_start, stem_end);

    let tail_transform = Transform2D::scale(height, height);
    let tail_path: Path = Path::new(|b| {
        b.move_to(Point::new(0.1003, 0.528));
        b.bezier_curve_to(
            Point::new(0.2944, 1.0012),
            Point::new(0.5062, 1.2791),
            Point::new(0.6412, 1.4329),
        );
        b.bezier_curve_to(
            Point::new(0.7478, 1.5543),
            Point::new(1.3322, 2.0942),
            Point::new(1., 2.8),
        );
        b.bezier_curve_to(
            Point::new(1.0586, 2.1117),
            Point::new(0.9148, 1.622),
            Point::new(0.1993, 1.32),
        );
        b.line_to(Point::new(0., 1.32));
        b.line_to(Point::new(0., -0.0115));
        b.close();
    });

    let stem_stroke = canvas::Stroke {
        width: height * 0.15,
        style: style.clone(),
        ..canvas::Stroke::default()
    };
    frame.stroke(&stem_path, stem_stroke);

    let fill = Fill {
        style: style.clone(),
        rule: Rule::NonZero,
    };

    for i in 0..num_tails {
        let t = if stem_direction == StemDirection::DOWN {
            tail_transform.then_scale(1., -1.)
        } else {
            tail_transform
        }
        .then_translate(Vector2D::new(
            stem_end.x,
            stem_end.y + sign * (i as f32) * height,
        ));

        frame.fill(&tail_path.transform(&t), fill);
    }
}
