use macros::ToJs;
use skia_safe as skia;

use crate::mem;
use crate::shapes::{Shadow, ShadowStyle};
use crate::{with_current_shape_mut, STATE};

const RAW_SHADOW_DATA_SIZE: usize = std::mem::size_of::<RawShadowData>();

#[derive(Debug, Clone, Copy, PartialEq, ToJs)]
#[repr(u8)]
#[allow(dead_code)]
pub enum RawShadowStyle {
    // NOTE: Odd naming to comply with cljs value
    DropShadow = 0,
    InnerShadow = 1,
}

impl From<u8> for RawShadowStyle {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<RawShadowStyle> for ShadowStyle {
    fn from(value: RawShadowStyle) -> Self {
        match value {
            RawShadowStyle::DropShadow => Self::Drop,
            RawShadowStyle::InnerShadow => Self::Inner,
        }
    }
}

/// Binary layout for a single shadow entry in the batched buffer.
///
/// | Offset | Size | Field   | Type |
/// |--------|------|---------|------|
/// | 0      | 4    | color   | u32  |
/// | 4      | 4    | blur    | f32  |
/// | 8      | 4    | spread  | f32  |
/// | 12     | 4    | x       | f32  |
/// | 16     | 4    | y       | f32  |
/// | 20     | 1    | style   | u8   |
/// | 21     | 1    | hidden  | u8   |
/// | 22     | 2    | padding | -    |
/// | Total  | 24   |         |      |
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
struct RawShadowData {
    color: u32,
    blur: f32,
    spread: f32,
    x: f32,
    y: f32,
    style: u8,
    hidden: u8,
    _padding: [u8; 2],
}

impl From<RawShadowData> for Shadow {
    fn from(raw: RawShadowData) -> Self {
        let color = skia::Color::new(raw.color);
        let style = RawShadowStyle::from(raw.style).into();
        Shadow::new(
            color,
            raw.blur,
            raw.spread,
            (raw.x, raw.y),
            style,
            raw.hidden != 0,
        )
    }
}

impl From<[u8; RAW_SHADOW_DATA_SIZE]> for RawShadowData {
    fn from(bytes: [u8; RAW_SHADOW_DATA_SIZE]) -> Self {
        unsafe { std::mem::transmute(bytes) }
    }
}

impl TryFrom<&[u8]> for RawShadowData {
    type Error = String;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let data: [u8; RAW_SHADOW_DATA_SIZE] = bytes
            .get(0..RAW_SHADOW_DATA_SIZE)
            .and_then(|slice| slice.try_into().ok())
            .ok_or("Invalid shadow data".to_string())?;
        Ok(RawShadowData::from(data))
    }
}

fn parse_shadows_from_bytes(buffer: &[u8], num_shadows: usize) -> Vec<Shadow> {
    buffer
        .chunks_exact(RAW_SHADOW_DATA_SIZE)
        .take(num_shadows)
        .map(|bytes| {
            RawShadowData::try_from(bytes)
                .expect("Invalid shadow data")
                .into()
        })
        .collect()
}

/// Batched shadow setter: reads all shadows from the shared memory buffer.
/// Buffer layout: [u8 count][3 bytes padding][N × 24-byte RawShadowData]
#[no_mangle]
pub extern "C" fn set_shape_shadows() {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        let bytes = mem::bytes();
        let num_shadows = bytes.first().copied().unwrap_or(0) as usize;
        let shadows = if num_shadows == 0 {
            vec![]
        } else {
            parse_shadows_from_bytes(&bytes[4..], num_shadows)
        };
        shape.set_shadows(shadows);
        mem::free_bytes();
    });
}

#[no_mangle]
pub extern "C" fn add_shape_shadow(
    raw_color: u32,
    blur: f32,
    spread: f32,
    x: f32,
    y: f32,
    raw_style: u8,
    hidden: bool,
) {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        let color = skia::Color::new(raw_color);
        let style = RawShadowStyle::from(raw_style).into();
        let shadow = Shadow::new(color, blur, spread, (x, y), style, hidden);
        shape.add_shadow(shadow);
    });
}

#[no_mangle]
pub extern "C" fn clear_shape_shadows() {
    with_current_shape_mut!(state, |shape: &mut Shape| {
        shape.clear_shadows();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_shadow_data_layout() {
        assert_eq!(RAW_SHADOW_DATA_SIZE, 24);
        assert_eq!(std::mem::align_of::<RawShadowData>(), 4);
    }

    #[test]
    fn test_raw_shadow_data_from_bytes() {
        let mut bytes = [0u8; RAW_SHADOW_DATA_SIZE];
        // color = 0xFF112233
        bytes[0..4].copy_from_slice(&0xFF112233_u32.to_le_bytes());
        // blur = 5.0
        bytes[4..8].copy_from_slice(&5.0_f32.to_le_bytes());
        // spread = 2.0
        bytes[8..12].copy_from_slice(&2.0_f32.to_le_bytes());
        // x = 10.0
        bytes[12..16].copy_from_slice(&10.0_f32.to_le_bytes());
        // y = 15.0
        bytes[16..20].copy_from_slice(&15.0_f32.to_le_bytes());
        // style = 0 (DropShadow)
        bytes[20] = 0;
        // hidden = 0 (false)
        bytes[21] = 0;

        let raw = RawShadowData::from(bytes);
        let shadow: Shadow = raw.into();

        assert_eq!(shadow.blur, 5.0);
        assert_eq!(shadow.spread, 2.0);
        assert_eq!(shadow.offset, (10.0, 15.0));
        assert!(!shadow.hidden());
    }
}
