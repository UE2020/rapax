
use glow::{
    CONSTANT_ALPHA, CONSTANT_COLOR, DST_ALPHA, DST_COLOR, ONE_MINUS_CONSTANT_ALPHA,
    ONE_MINUS_CONSTANT_COLOR, ONE_MINUS_DST_ALPHA, ONE_MINUS_DST_COLOR, ONE_MINUS_SRC_ALPHA,
    ONE_MINUS_SRC_COLOR, SRC_ALPHA, SRC_COLOR,
};

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum BlendFactor {
    ConstantAlpha = CONSTANT_ALPHA,
    ConstantColor = CONSTANT_COLOR,
    DestinationAlpha = DST_ALPHA,
    DestinationColor = DST_COLOR,
    OneMinusConstantAlpha = ONE_MINUS_CONSTANT_ALPHA,
    OneMinusConstantColor = ONE_MINUS_CONSTANT_COLOR,
    OneMinusDestinationAlpha = ONE_MINUS_DST_ALPHA,
    OneMinusDestinationColor = ONE_MINUS_DST_COLOR,
    OneMinusSourceAlpha = ONE_MINUS_SRC_ALPHA,
    OneMinusSourceColor = ONE_MINUS_SRC_COLOR,
    SourceAlpha = SRC_ALPHA,
    SourceColor = SRC_COLOR,
}