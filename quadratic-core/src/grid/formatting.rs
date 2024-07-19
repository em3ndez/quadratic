use super::{block::SameValue, Column, ColumnData};
use crate::RunLengthEncoding;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum_macros::{Display, EnumString};

/// Array of a single cell formatting attribute.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CellFmtArray {
    Align(RunLengthEncoding<Option<CellAlign>>),
    Wrap(RunLengthEncoding<Option<CellWrap>>),
    NumericFormat(RunLengthEncoding<Option<NumericFormat>>),
    NumericDecimals(RunLengthEncoding<Option<i16>>),
    NumericCommas(RunLengthEncoding<Option<bool>>),
    Bold(RunLengthEncoding<Option<bool>>),
    Italic(RunLengthEncoding<Option<bool>>),
    TextColor(RunLengthEncoding<Option<String>>),
    FillColor(RunLengthEncoding<Option<String>>),
    RenderSize(RunLengthEncoding<Option<RenderSize>>),
}

/// Cell formatting attribute.
pub trait CellFmtAttr {
    type Value: Serialize + for<'d> Deserialize<'d> + fmt::Debug + Clone + Eq;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>>;
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>>;
}

impl CellFmtAttr for CellAlign {
    type Value = Self;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.align
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.align
    }
}
impl CellFmtAttr for CellWrap {
    type Value = Self;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.wrap
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.wrap
    }
}
impl CellFmtAttr for NumericFormat {
    type Value = Self;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.numeric_format
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.numeric_format
    }
}
pub struct NumericDecimals;
impl CellFmtAttr for NumericDecimals {
    type Value = i16;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.numeric_decimals
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.numeric_decimals
    }
}

pub struct NumericCommas;
impl CellFmtAttr for NumericCommas {
    type Value = bool;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.numeric_commas
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.numeric_commas
    }
}

pub struct Bold;
impl CellFmtAttr for Bold {
    type Value = bool;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.bold
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.bold
    }
}
pub struct Italic;
impl CellFmtAttr for Italic {
    type Value = bool;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.italic
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.italic
    }
}
pub struct TextColor;
impl CellFmtAttr for TextColor {
    type Value = String;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.text_color
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.text_color
    }
}
pub struct FillColor;
impl CellFmtAttr for FillColor {
    type Value = String;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.fill_color
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.fill_color
    }
}

impl CellFmtAttr for RenderSize {
    type Value = Self;
    fn column_data_ref(column: &Column) -> &ColumnData<SameValue<Self::Value>> {
        &column.render_size
    }
    fn column_data_mut(column: &mut Column) -> &mut ColumnData<SameValue<Self::Value>> {
        &mut column.render_size
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, Display, EnumString)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum CellAlign {
    Center,
    Left,
    Right,
}

#[derive(
    Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Display, EnumString,
)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum CellWrap {
    #[default]
    Overflow,
    Wrap,
    Clip,
}

impl CellWrap {
    pub fn as_css_string(&self) -> &'static str {
        match self {
            CellWrap::Overflow => "overflow: visible; white-space: nowrap;",
            CellWrap::Wrap => "overflow: hidden; white-space: normal; word-wrap: break-word;",
            CellWrap::Clip => "overflow: hidden; white-space: clip;",
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
pub struct NumericFormat {
    #[serde(rename = "type")]
    pub kind: NumericFormatKind,
    pub symbol: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
/// Measures DOM element size in pixels.
pub struct RenderSize {
    pub w: String,
    pub h: String,
}

#[derive(
    Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, Copy,
)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
#[serde(rename_all = "UPPERCASE")]
#[strum(ascii_case_insensitive)]
pub enum NumericFormatKind {
    #[default]
    Number,
    Currency, // { symbol: String }, // TODO: would be nice if this were just a single char (and it could be)
    Percentage,
    Exponential,
}
