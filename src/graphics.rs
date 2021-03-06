use glutin::dpi::{PhysicalPosition, PhysicalSize};
use std::fmt::{Display, Formatter};

pub type Color = [u8; 3];

pub trait ColorExt {
    const WHITE: [u8; 3] = [0xff, 0xff, 0xff];
    const BLACK: [u8; 3] = [0x00, 0x00, 0x00];

    fn grey(level: f32) -> Color {
        [
            (level * 0xff as f32) as u8,
            (level * 0xff as f32) as u8,
            (level * 0xff as f32) as u8,
        ]
    }
}

impl ColorExt for Color {}

pub fn circle_points(radius: f32, num_segments: usize) -> Vec<f32> {
    let mut segments = Vec::with_capacity(num_segments);

    let mut angle = 0.0;
    let segments_f32 = num_segments as f32;
    for _ in 0..num_segments {
        let d_theta = std::f32::consts::TAU / segments_f32;
        angle += d_theta;
        let (x, y) = angle.sin_cos();
        segments.push(x * radius);
        segments.push(y * radius);
    }

    segments
}

pub fn view_matrix(zoom: f32, size: PhysicalSize<u32>, origin: StrokePoint) -> glam::Mat4 {
    let PhysicalSize { width, height } = size;
    let xform = stroke_to_gl(width, height, zoom, origin);
    glam::Mat4::from_scale_rotation_translation(
        glam::vec3(zoom / width as f32, zoom / height as f32, 1.0),
        glam::Quat::IDENTITY,
        glam::vec3(xform.x, xform.y, 0.0),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct GlPos {
    pub x: f32,
    pub y: f32,
}

pub fn physical_position_to_gl(width: u32, height: u32, pos: PhysicalPosition<f64>) -> GlPos {
    GlPos {
        x: (2.0 * pos.x as f32) / width as f32 - 1.0,
        y: -((2.0 * pos.y as f32) / height as f32 - 1.0),
    }
}

pub fn gl_to_physical_position(width: u32, height: u32, pos: GlPos) -> PhysicalPosition<f64> {
    PhysicalPosition {
        x: (pos.x as f64 + 1.0) * width as f64 / 2.0,
        y: (-pos.y as f64 + 1.0) * height as f64 / 2.0,
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
}

pub fn gl_to_stroke(width: u32, height: u32, zoom: f32, gl: GlPos) -> StrokePoint {
    StrokePoint {
        x: gl.x * width as f32 / zoom,
        y: gl.y * height as f32 / zoom,
    }
}

pub fn stroke_to_gl(width: u32, height: u32, zoom: f32, point: StrokePoint) -> GlPos {
    GlPos {
        x: point.x * zoom / width as f32,
        y: point.y * zoom / height as f32,
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct StrokePos {
    pub x: f32,
    pub y: f32,
}

pub fn xform_point_to_pos(gis: StrokePoint, stroke: StrokePoint) -> StrokePos {
    let x = stroke.x - gis.x;
    let y = stroke.y - gis.y;
    StrokePos { x, y }
}

impl Display for GlPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.02},{:.02}", self.x, self.y)
    }
}

impl Display for StrokePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.02},{:.02}", self.x, self.y)
    }
}

impl Display for StrokePos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.02},{:.02}", self.x, self.y)
    }
}
