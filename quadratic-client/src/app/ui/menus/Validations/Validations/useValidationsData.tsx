import { hasPermissionToEditFile } from '@/app/actions';
import { editorInteractionStateAtom } from '@/app/atoms/editorInteractionStateAtom';
import { events } from '@/app/events/events';
import { sheets } from '@/app/grid/controller/Sheets';
import { Validation } from '@/app/quadratic-core-types';
import { quadraticCore } from '@/app/web-workers/quadraticCore/quadraticCore';
import { useCallback, useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';

export interface ValidationsData {
  sheetId: string;
  validations: Validation[];
  deleteValidation: (validationId: string) => void;
  readOnly: boolean;
}

export const useValidationsData = (): ValidationsData => {
  const { permissions } = useRecoilValue(editorInteractionStateAtom);
  const readOnly = !hasPermissionToEditFile(permissions);

  const [sheetId, setSheetId] = useState(sheets.sheet.id);
  useEffect(() => {
    const updateSheet = () => {
      setSheetId((current: string) => {
        if (current !== sheets.sheet.id) {
          setValidations([...sheets.sheet.validations]);
          return sheets.sheet.id;
        }
        return current;
      });
    };
    events.on('changeSheet', updateSheet);
    return () => {
      events.off('changeSheet', updateSheet);
    };
  });

  // we make a copy of validations from sheet so we can delete pending ones
  // without affecting the sheet.
  const [validations, setValidations] = useState([...sheets.sheet.validations]);
  useEffect(() => {
    const updateValidations = (incomingSheetId: string, validations: Validation[]) => {
      if (incomingSheetId === sheetId) {
        setValidations(validations);
      }
    };
    events.on('sheetValidations', updateValidations);
    return () => {
      events.off('sheetValidations', updateValidations);
    };
  }, [sheetId]);

  const deleteValidation = useCallback(
    (validationId: string) => {
      quadraticCore.removeValidation(sheetId, validationId, sheets.getCursorPosition());
      setValidations((prev) => prev.filter((v) => v.id !== validationId));
    },
    [sheetId]
  );

  return {
    sheetId,
    validations,
    deleteValidation,
    readOnly,
  };
};