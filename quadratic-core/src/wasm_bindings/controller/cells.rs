use std::str::FromStr;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{controller::GridController, grid::SheetId, selection::Selection, Pos, Rect};

#[wasm_bindgen]
impl GridController {
    /// Sets a cell value given as a [`CellValue`].
    ///
    /// Returns a [`TransactionSummary`].
    #[wasm_bindgen(js_name = "setCellValue")]
    pub fn js_set_cell_value(
        &mut self,
        sheet_id: String,
        x: i32,
        y: i32,
        value: String,
        cursor: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let pos = Pos {
            x: x as i64,
            y: y as i64,
        };
        if let Ok(sheet_id) = SheetId::from_str(&sheet_id) {
            Ok(serde_wasm_bindgen::to_value(&self.set_cell_value(
                pos.to_sheet_pos(sheet_id),
                value,
                cursor,
            ))?)
        } else {
            Err(JsValue::from_str("Invalid sheet id"))
        }
    }

    /// changes the decimal places
    #[wasm_bindgen(js_name = "setCellNumericDecimals")]
    pub fn js_set_cell_numeric_decimals(
        &mut self,
        sheet_id: String,
        source: String,
        rect: String,
        delta: isize,
        cursor: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let source: Pos =
            serde_json::from_str(&source).map_err(|_| JsValue::from_str("Invalid source"))?;
        let rect: Rect =
            serde_json::from_str(&rect).map_err(|_| JsValue::from_str("Invalid rect"))?;
        if let Ok(sheet_id) = SheetId::from_str(&sheet_id) {
            Ok(serde_wasm_bindgen::to_value(&self.change_decimal_places(
                source.to_sheet_pos(sheet_id),
                rect.to_sheet_rect(sheet_id),
                delta,
                cursor,
            ))?)
        } else {
            Err(JsValue::from_str("Invalid sheet id"))
        }
    }

    /// gets an editable string for a cell
    ///
    /// returns a string
    #[wasm_bindgen(js_name = "getEditCell")]
    pub fn js_get_cell_edit(&self, sheet_id: String, pos: String) -> Result<String, JsValue> {
        let pos = serde_json::from_str(&pos).map_err(|_| JsValue::UNDEFINED)?;
        let sheet = self
            .try_sheet_from_string_id(sheet_id)
            .ok_or(JsValue::UNDEFINED)?;
        if let Some(value) = sheet.cell_value(pos) {
            Ok(value.to_edit())
        } else {
            Ok(String::from(""))
        }
    }

    /// Deletes a region of cells.
    #[wasm_bindgen(js_name = "deleteCellValues")]
    pub fn js_delete_cell_values(
        &mut self,
        selection: String,
        cursor: Option<String>,
    ) -> Result<(), JsValue> {
        let selection =
            Selection::from_str(&selection).map_err(|_| JsValue::from_str("Invalid selection"))?;
        self.delete_cells(&selection, cursor);
        Ok(())
    }
}
