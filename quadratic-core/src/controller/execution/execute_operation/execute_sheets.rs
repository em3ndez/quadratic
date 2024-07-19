use crate::{
    controller::{
        active_transactions::pending_transaction::PendingTransaction,
        operations::operation::Operation, GridController,
    },
    grid::{file::sheet_schema::export_sheet, Sheet, SheetId},
};
use lexicon_fractional_index::key_between;

impl GridController {
    pub(crate) fn execute_add_sheet(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::AddSheet { sheet } = op {
            let sheet_id = sheet.id;
            if self.grid.try_sheet(sheet_id).is_some() {
                // sheet already exists (unlikely but possible if this operation is run twice)
                return;
            }
            let sheet_id = self.grid.add_sheet(Some(sheet.clone()));

            self.send_add_sheet(sheet_id, transaction);

            transaction
                .forward_operations
                .push(Operation::AddSheetSchema {
                    schema: export_sheet(&sheet),
                });
            transaction
                .reverse_operations
                .insert(0, Operation::DeleteSheet { sheet_id });
        }
    }

    pub(crate) fn execute_add_sheet_schema(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::AddSheetSchema { schema } = op {
            if let Ok(sheet) = schema.into_latest() {
                if self.grid.try_sheet(sheet.id).is_some() {
                    // sheet already exists (unlikely but possible if this operation is run twice)
                    return;
                }
                let sheet_id = sheet.id;
                self.grid.add_sheet(Some(sheet));

                self.send_add_sheet(sheet_id, transaction);

                transaction
                    .forward_operations
                    .push(Operation::AddSheetSchema { schema });
                transaction
                    .reverse_operations
                    .insert(0, Operation::DeleteSheet { sheet_id });
            }
        }
    }

    pub(crate) fn execute_delete_sheet(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::DeleteSheet { sheet_id } = op {
            let Some(deleted_sheet) = self.grid.remove_sheet(sheet_id) else {
                // sheet was already deleted
                return;
            };

            // create a sheet if we deleted the last one (only for user actions)
            if transaction.is_user() && self.sheet_ids().is_empty() {
                let new_first_sheet_id = SheetId::new();
                let name = String::from("Sheet 1");
                let order = self.grid.end_order();
                let new_first_sheet = Sheet::new(new_first_sheet_id, name, order);
                self.grid.add_sheet(Some(new_first_sheet.clone()));
                transaction
                    .forward_operations
                    .push(Operation::DeleteSheet { sheet_id });
                transaction.forward_operations.push(Operation::AddSheet {
                    sheet: new_first_sheet,
                });
                transaction.reverse_operations.insert(
                    0,
                    Operation::AddSheetSchema {
                        schema: export_sheet(&deleted_sheet),
                    },
                );
                transaction.reverse_operations.insert(
                    0,
                    Operation::DeleteSheet {
                        sheet_id: new_first_sheet_id,
                    },
                );

                // if that's the last sheet, then we created a new one and we have to let the workers know
                self.send_add_sheet(new_first_sheet_id, transaction);
            } else {
                transaction
                    .forward_operations
                    .push(Operation::DeleteSheet { sheet_id });
                transaction.reverse_operations.insert(
                    0,
                    Operation::AddSheetSchema {
                        schema: export_sheet(&deleted_sheet),
                    },
                );

                // otherwise we need to send the deleted sheet information to the workers
            }
            self.send_delete_sheet(sheet_id, transaction);
        }
    }

    pub(crate) fn execute_reorder_sheet(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::ReorderSheet { target, order } = op {
            let old_first = self.grid.first_sheet_id();
            let Some(sheet) = self.try_sheet_mut(target) else {
                // sheet may have been deleted
                return;
            };
            let original_order = sheet.order.clone();
            sheet.order.clone_from(&order);
            self.grid.move_sheet(target, order.clone());

            if old_first != self.grid.first_sheet_id() {
                transaction.generate_thumbnail = true;
            }
            transaction
                .forward_operations
                .push(Operation::ReorderSheet { target, order });
            transaction.reverse_operations.insert(
                0,
                Operation::ReorderSheet {
                    target,
                    order: original_order,
                },
            );

            self.send_sheet_info(target);
        }
    }

    pub(crate) fn execute_set_sheet_name(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::SetSheetName { sheet_id, name } = op {
            let Some(sheet) = self.try_sheet_mut(sheet_id) else {
                // sheet may have been deleted
                return;
            };
            let old_name = sheet.name.clone();
            sheet.name.clone_from(&name);

            transaction
                .forward_operations
                .push(Operation::SetSheetName { sheet_id, name });
            transaction.reverse_operations.insert(
                0,
                Operation::SetSheetName {
                    sheet_id,
                    name: old_name,
                },
            );

            self.send_sheet_info(sheet_id);
        }
    }

    pub(crate) fn execute_set_sheet_color(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::SetSheetColor { sheet_id, color } = op {
            let Some(sheet) = self.try_sheet_mut(sheet_id) else {
                // sheet may have been deleted
                return;
            };
            let old_color = sheet.color.clone();
            sheet.color.clone_from(&color);

            transaction
                .forward_operations
                .push(Operation::SetSheetColor { sheet_id, color });
            transaction.reverse_operations.insert(
                0,
                Operation::SetSheetColor {
                    sheet_id,
                    color: old_color,
                },
            );

            self.send_sheet_info(sheet_id);
        }
    }

    pub(crate) fn execute_duplicate_sheet(
        &mut self,
        transaction: &mut PendingTransaction,
        op: Operation,
    ) {
        if let Operation::DuplicateSheet {
            sheet_id,
            new_sheet_id,
        } = op
        {
            let Some(sheet) = self.try_sheet(sheet_id) else {
                // sheet may have been deleted
                return;
            };
            let mut new_sheet = sheet.clone();
            new_sheet.id = new_sheet_id;
            let right = self.grid.next_sheet(sheet_id);
            let right_order = right.map(|right| right.order.clone());
            if let Ok(order) = key_between(&Some(sheet.order.clone()), &right_order) {
                new_sheet.order = order;
            };
            let name = format!("{} Copy", sheet.name);
            let sheet_names = self.sheet_names();
            if !sheet_names.contains(&name.as_str()) {
                new_sheet.name = name;
            } else {
                new_sheet.name = crate::util::unused_name(&name, &self.sheet_names());
            }
            self.grid.add_sheet(Some(new_sheet));

            self.send_add_sheet(new_sheet_id, transaction);

            transaction
                .forward_operations
                .push(Operation::DuplicateSheet {
                    sheet_id,
                    new_sheet_id,
                });
            transaction.reverse_operations.insert(
                0,
                Operation::DeleteSheet {
                    sheet_id: new_sheet_id,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        controller::GridController,
        grid::CodeCellLanguage,
        wasm_bindings::{controller::sheet_info::SheetInfo, js::expect_js_call},
        SheetPos,
    };
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_add_sheet() {
        let mut gc = GridController::test();
        gc.add_sheet(None);
        assert_eq!(gc.grid.sheets().len(), 2);
        let sheet_id = gc.sheet_ids()[1];
        let sheet = gc.sheet(sheet_id);
        let sheet_info = SheetInfo::from(sheet);
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            true,
        );

        // was jsAdSheet called with the right stuff
        gc.undo(None);
        assert_eq!(gc.grid.sheets().len(), 1);
        expect_js_call("jsDeleteSheet", format!("{},{}", sheet_id, true), true);
    }

    #[test]
    #[serial]
    fn test_delete_sheet() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        gc.delete_sheet(sheet_id, None);
        assert_eq!(gc.grid.sheets().len(), 1);
        expect_js_call("jsDeleteSheet", format!("{},{}", sheet_id, true), true);
        let new_sheet_id = gc.sheet_ids()[0];

        gc.undo(None);
        assert_eq!(gc.grid.sheets().len(), 1);
        assert_eq!(gc.grid.sheets()[0].id, sheet_id);
        let sheet = gc.sheet(sheet_id);
        let sheet_info = SheetInfo::from(sheet);
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            false,
        );
        expect_js_call("jsDeleteSheet", format!("{},{}", new_sheet_id, true), true);
    }

    #[test]
    #[serial]
    fn test_execute_operation_set_sheet_name() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let name = "new name".to_string();
        gc.set_sheet_name(sheet_id, name.clone(), None);
        assert_eq!(gc.grid.sheets()[0].name, name);

        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );

        gc.undo(None);
        assert_eq!(gc.grid.sheets()[0].name, "Sheet 1".to_string());
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );
    }

    #[test]
    #[serial]
    fn test_set_sheet_color() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let color = Some("red".to_string());
        gc.set_sheet_color(sheet_id, color.clone(), None);
        assert_eq!(gc.grid.sheets()[0].color, color);
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );

        gc.undo(None);
        assert_eq!(gc.grid.sheets()[0].color, None);
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );
    }

    #[test]
    #[serial]
    fn test_sheet_reorder() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        // Sheet 1, Sheet 2
        gc.add_sheet(None);
        assert_eq!(gc.grid.sheets().len(), 2);
        let sheet_id2 = gc.sheet_ids()[1];
        assert_eq!(gc.grid.sheets()[0].id, sheet_id);
        assert_eq!(gc.grid.sheets()[1].id, sheet_id2);

        // Sheet 2, Sheet 1
        gc.move_sheet(sheet_id, None, None);
        assert_eq!(gc.grid.sheets()[0].id, sheet_id2);
        assert_eq!(gc.grid.sheets()[1].id, sheet_id);

        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );

        // Sheet 1, Sheet 2
        gc.undo(None);
        assert_eq!(gc.grid.sheets()[0].id, sheet_id);
        assert_eq!(gc.grid.sheets()[1].id, sheet_id2);
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );

        gc.move_sheet(sheet_id2, Some(sheet_id), None);
        assert_eq!(gc.grid.sheets()[0].id, sheet_id2);
        assert_eq!(gc.grid.sheets()[1].id, sheet_id);
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id2));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );

        gc.undo(None);
        assert_eq!(gc.grid.sheets()[0].id, sheet_id);
        assert_eq!(gc.grid.sheets()[1].id, sheet_id2);
        let sheet_info = SheetInfo::from(gc.sheet(sheet_id2));
        expect_js_call(
            "jsSheetInfoUpdate",
            serde_json::to_string(&sheet_info).unwrap(),
            true,
        );
    }

    #[test]
    #[serial]
    fn duplicate_sheet() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        gc.set_code_cell(
            SheetPos {
                sheet_id,
                x: 0,
                y: 0,
            },
            CodeCellLanguage::Formula,
            "10 + 10".to_string(),
            None,
        );

        gc.duplicate_sheet(sheet_id, None);
        assert_eq!(gc.grid.sheets().len(), 2);
        assert_eq!(gc.grid.sheets()[1].name, "Sheet 1 Copy");
        let duplicated_sheet_id = gc.grid.sheets()[1].id;
        let sheet_info = SheetInfo::from(gc.sheet(duplicated_sheet_id));
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            true,
        );

        gc.undo(None);
        assert_eq!(gc.grid.sheets().len(), 1);
        expect_js_call(
            "jsDeleteSheet",
            format!("{},{}", duplicated_sheet_id, true),
            true,
        );

        gc.duplicate_sheet(sheet_id, None);
        assert_eq!(gc.grid.sheets().len(), 2);
        let duplicated_sheet_id2 = gc.grid.sheets()[1].id;
        let sheet_info = SheetInfo::from(gc.sheet(duplicated_sheet_id2));
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            true,
        );

        gc.duplicate_sheet(sheet_id, None);
        assert_eq!(gc.grid.sheets().len(), 3);
        assert_eq!(gc.grid.sheets()[1].name, "Sheet 1 Copy 1");
        assert_eq!(gc.grid.sheets()[2].name, "Sheet 1 Copy");
        let duplicated_sheet_id3 = gc.grid.sheets()[1].id;
        let sheet_info = SheetInfo::from(gc.sheet(duplicated_sheet_id3));
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            true,
        );

        gc.undo(None);
        assert_eq!(gc.grid.sheets().len(), 2);
        assert_eq!(gc.grid.sheets()[1].name, "Sheet 1 Copy");
        expect_js_call(
            "jsDeleteSheet",
            format!("{},{}", duplicated_sheet_id3, true),
            true,
        );

        gc.redo(None);
        assert_eq!(gc.grid.sheets().len(), 3);
        assert_eq!(gc.grid.sheets()[1].name, "Sheet 1 Copy 1");
        let sheet_info = SheetInfo::from(gc.sheet(duplicated_sheet_id3));
        expect_js_call(
            "jsAddSheet",
            format!("{},{}", serde_json::to_string(&sheet_info).unwrap(), true),
            true,
        );
    }
}
