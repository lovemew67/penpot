use macros::ToJs;

use crate::mem;
use crate::shapes::{self, Fill, StrokeCap, StrokeKind, StrokeStyle};
use crate::with_current_shape_mut;
use crate::STATE;

use super::fills::RawFillData;

const RAW_FILL_DATA_SIZE: usize = std::mem::size_of::<RawFillData>();

/// Binary layout per stroke entry (matches STROKE-ENTRY-U8-SIZE on CLJS side):
///   byte 0:   kind      (0=center, 1=inner, 2=outer)
///   byte 1:   style     (0=solid, 1=dotted, 2=dashed, 3=mixed)
///   byte 2:   cap_start
///   byte 3:   cap_end
///   bytes 4-7: width    (f32, little-endian)
///   bytes 8..: fill data (RAW_FILL_DATA_SIZE bytes)
const STROKE_ENTRY_SIZE: usize = 8 + RAW_FILL_DATA_SIZE;

#[derive(Debug, Clone, PartialEq, Copy, ToJs)]
#[repr(u8)]
#[allow(dead_code)]
pub enum RawStrokeStyle {
    Solid = 0,
    Dotted = 1,
    Dashed = 2,
    Mixed = 3,
}

impl From<u8> for RawStrokeStyle {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<RawStrokeStyle> for StrokeStyle {
    fn from(value: RawStrokeStyle) -> Self {
        match value {
            RawStrokeStyle::Solid => StrokeStyle::Solid,
            RawStrokeStyle::Dotted => StrokeStyle::Dotted,
            RawStrokeStyle::Dashed => StrokeStyle::Dashed,
            RawStrokeStyle::Mixed => StrokeStyle::Mixed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ToJs)]
#[repr(u8)]
#[allow(dead_code)]
pub enum RawStrokeCap {
    None = 0,
    LineArrow = 1,
    TriangleArrow = 2,
    SquareMarker = 3,
    CircleMarker = 4,
    DiamondMarker = 5,
    Round = 6,
    Square = 7,
}

impl From<u8> for RawStrokeCap {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl TryFrom<RawStrokeCap> for StrokeCap {
    type Error = ();

    fn try_from(value: RawStrokeCap) -> Result<Self, Self::Error> {
        match value {
            RawStrokeCap::None => Err(()),
            RawStrokeCap::LineArrow => Ok(StrokeCap::LineArrow),
            RawStrokeCap::TriangleArrow => Ok(StrokeCap::TriangleArrow),
            RawStrokeCap::SquareMarker => Ok(StrokeCap::SquareMarker),
            RawStrokeCap::CircleMarker => Ok(StrokeCap::CircleMarker),
            RawStrokeCap::DiamondMarker => Ok(StrokeCap::DiamondMarker),
            RawStrokeCap::Round => Ok(StrokeCap::Round),
            RawStrokeCap::Square => Ok(StrokeCap::Square),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum RawStrokeKind {
    Center = 0,
    Inner = 1,
    Outer = 2,
}

impl From<u8> for RawStrokeKind {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<RawStrokeKind> for StrokeKind {
    fn from(value: RawStrokeKind) -> Self {
        match value {
            RawStrokeKind::Center => StrokeKind::Center,
            RawStrokeKind::Inner => StrokeKind::Inner,
            RawStrokeKind::Outer => StrokeKind::Outer,
        }
    }
}

fn parse_stroke_from_bytes(bytes: &[u8]) -> shapes::Stroke {
    let kind = RawStrokeKind::from(bytes[0]);
    let style = RawStrokeStyle::from(bytes[1]);
    let cap_start = RawStrokeCap::from(bytes[2]);
    let cap_end = RawStrokeCap::from(bytes[3]);
    let width = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let fill_bytes = &bytes[8..8 + RAW_FILL_DATA_SIZE];
    let raw_fill = RawFillData::try_from(fill_bytes).expect("Invalid stroke fill data");
    let fill: Fill = raw_fill.into();

    shapes::Stroke {
        fill,
        width,
        style: style.into(),
        cap_start: cap_start.try_into().ok(),
        cap_end: cap_end.try_into().ok(),
        kind: kind.into(),
    }
}

fn parse_strokes_from_bytes(buffer: &[u8], num_strokes: usize) -> Vec<shapes::Stroke> {
    buffer
        .chunks_exact(STROKE_ENTRY_SIZE)
        .take(num_strokes)
        .map(parse_stroke_from_bytes)
        .collect()
}

/// Batched stroke setter: reads all strokes from a single shared-memory buffer.
///
/// Buffer layout:
///   byte 0:      number of strokes (u8)
///   bytes 1-3:   padding (reserved)
///   bytes 4..:   N × STROKE_ENTRY_SIZE stroke entries
#[no_mangle]
pub extern "C" fn set_shape_strokes() {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        let bytes = mem::bytes();
        let num_strokes = bytes.first().copied().unwrap_or(0) as usize;
        let strokes = parse_strokes_from_bytes(&bytes[4..], num_strokes);
        shape.set_strokes(strokes);
        mem::free_bytes();
    });
}

#[no_mangle]
pub extern "C" fn add_shape_center_stroke(width: f32, style: u8, cap_start: u8, cap_end: u8) {
    let stroke_style = RawStrokeStyle::from(style);
    let cap_start = RawStrokeCap::from(cap_start);
    let cap_end = RawStrokeCap::from(cap_end);

    with_current_shape_mut!(state, |shape: &mut Shape| {
        shape.add_stroke(shapes::Stroke::new_center_stroke(
            width,
            stroke_style.into(),
            cap_start.try_into().ok(),
            cap_end.try_into().ok(),
        ));
    });
}

#[no_mangle]
pub extern "C" fn add_shape_inner_stroke(width: f32, style: u8, cap_start: u8, cap_end: u8) {
    let stroke_style = RawStrokeStyle::from(style);
    let cap_start = RawStrokeCap::from(cap_start);
    let cap_end = RawStrokeCap::from(cap_end);

    with_current_shape_mut!(state, |shape: &mut Shape| {
        shape.add_stroke(shapes::Stroke::new_inner_stroke(
            width,
            stroke_style.into(),
            cap_start.try_into().ok(),
            cap_end.try_into().ok(),
        ));
    });
}

#[no_mangle]
pub extern "C" fn add_shape_outer_stroke(width: f32, style: u8, cap_start: u8, cap_end: u8) {
    let stroke_style = RawStrokeStyle::from(style);
    let cap_start = RawStrokeCap::from(cap_start);
    let cap_end = RawStrokeCap::from(cap_end);

    with_current_shape_mut!(state, |shape: &mut Shape| {
        shape.add_stroke(shapes::Stroke::new_outer_stroke(
            width,
            stroke_style.into(),
            cap_start.try_into().ok(),
            cap_end.try_into().ok(),
        ));
    });
}

#[no_mangle]
pub extern "C" fn add_shape_stroke_fill() {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        let bytes = mem::bytes();
        let raw_fill = super::fills::RawFillData::try_from(&bytes[..]).expect("Invalid fill data");
        shape
            .set_stroke_fill(raw_fill.into())
            .expect("could not add stroke fill");
    });
}

#[no_mangle]
pub extern "C" fn clear_shape_strokes() {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        shape.clear_strokes();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stroke_entry_size() {
        // STROKE_ENTRY_SIZE = 8 bytes header + RAW_FILL_DATA_SIZE bytes fill
        assert_eq!(STROKE_ENTRY_SIZE, 8 + RAW_FILL_DATA_SIZE);
    }

    #[test]
    fn test_parse_stroke_center_solid() {
        let mut entry = vec![0u8; STROKE_ENTRY_SIZE];
        // kind = Center (0)
        entry[0] = 0;
        // style = Solid (0)
        entry[1] = 0;
        // cap_start = None (0)
        entry[2] = 0;
        // cap_end = None (0)
        entry[3] = 0;
        // width = 5.0 (f32 LE)
        entry[4..8].copy_from_slice(&5.0f32.to_le_bytes());
        // fill: solid (tag byte 0x00 at offset 8, color at offset 12)
        entry[8] = 0x00;
        entry[12..16].copy_from_slice(&0xff00ff00_u32.to_le_bytes());

        let stroke = parse_stroke_from_bytes(&entry);

        assert_eq!(stroke.kind, StrokeKind::Center);
        assert_eq!(stroke.style, StrokeStyle::Solid);
        assert_eq!(stroke.cap_start, None);
        assert_eq!(stroke.cap_end, None);
        assert_eq!(stroke.width, 5.0);
    }

    #[test]
    fn test_parse_stroke_inner_dashed_with_caps() {
        let mut entry = vec![0u8; STROKE_ENTRY_SIZE];
        // kind = Inner (1)
        entry[0] = 1;
        // style = Dashed (2)
        entry[1] = 2;
        // cap_start = LineArrow (1)
        entry[2] = 1;
        // cap_end = Round (6)
        entry[3] = 6;
        // width = 3.5 (f32 LE)
        entry[4..8].copy_from_slice(&3.5f32.to_le_bytes());
        // fill: solid
        entry[8] = 0x00;
        entry[12..16].copy_from_slice(&0xffaabbcc_u32.to_le_bytes());

        let stroke = parse_stroke_from_bytes(&entry);

        assert_eq!(stroke.kind, StrokeKind::Inner);
        assert_eq!(stroke.style, StrokeStyle::Dashed);
        assert_eq!(stroke.cap_start, Some(StrokeCap::LineArrow));
        assert_eq!(stroke.cap_end, Some(StrokeCap::Round));
        assert_eq!(stroke.width, 3.5);
    }

    #[test]
    fn test_parse_multiple_strokes() {
        let mut buffer = vec![0u8; 2 * STROKE_ENTRY_SIZE];

        // First stroke: center solid
        buffer[0] = 0; // center
        buffer[1] = 0; // solid
        buffer[4..8].copy_from_slice(&2.0f32.to_le_bytes());
        buffer[8] = 0x00; // solid fill
        buffer[12..16].copy_from_slice(&0xff000000_u32.to_le_bytes());

        // Second stroke: outer dotted
        let off = STROKE_ENTRY_SIZE;
        buffer[off] = 2; // outer
        buffer[off + 1] = 1; // dotted
        buffer[off + 4..off + 8].copy_from_slice(&4.0f32.to_le_bytes());
        buffer[off + 8] = 0x00; // solid fill
        buffer[off + 12..off + 16].copy_from_slice(&0xff0000ff_u32.to_le_bytes());

        let strokes = parse_strokes_from_bytes(&buffer, 2);

        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].kind, StrokeKind::Center);
        assert_eq!(strokes[0].width, 2.0);
        assert_eq!(strokes[1].kind, StrokeKind::Outer);
        assert_eq!(strokes[1].style, StrokeStyle::Dotted);
        assert_eq!(strokes[1].width, 4.0);
    }
}
