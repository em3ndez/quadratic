import { Alert, Button, Paper, Stack, useTheme } from '@mui/material';
import React, { useState } from 'react';
import { isMobile } from 'react-device-detect';
import { Link, useSubmit } from 'react-router-dom';
import { useRecoilValue } from 'recoil';
import { duplicateFileAction } from '../../actions';
import { PermissionSchema } from '../../api/types';
import { editorInteractionStateAtom } from '../../atoms/editorInteractionStateAtom';
import { ROUTES } from '../../constants/routes';
import { useFileContext } from './FileProvider';
const { ANONYMOUS, VIEWER, OWNER, EDITOR } = PermissionSchema.enum;

export function PermissionOverlay() {
  const [isOpen, setIsOpen] = useState<boolean>(true);
  const { permission } = useRecoilValue(editorInteractionStateAtom);
  const { name } = useFileContext();
  const theme = useTheme();
  const submit = useSubmit();

  if ((permission === OWNER || permission === EDITOR) && isMobile && isOpen) {
    return (
      <Wrapper>
        <Alert variant="outlined" severity="info" sx={{ width: '100%' }} onClose={() => setIsOpen(false)}>
          <strong>Read-only on mobile.</strong> Open on desktop to edit cells and run code.
        </Alert>
      </Wrapper>
    );
  }

  if (permission === ANONYMOUS) {
    return (
      <Wrapper>
        <Alert
          variant="outlined"
          severity="info"
          sx={{ width: '100%' }}
          action={
            <Stack direction="row" gap={theme.spacing(1)}>
              <Button component={Link} to={ROUTES.LOGIN_WITH_REDIRECT()} variant="outlined" size="small">
                Log in
              </Button>
              <Button
                component={Link}
                to={ROUTES.SIGNUP_WITH_REDIRECT()}
                variant="contained"
                size="small"
                disableElevation
              >
                Sign up
              </Button>
            </Stack>
          }
        >
          <strong>Welcome to Quadratic.</strong> You must log in to edit this file.
        </Alert>
      </Wrapper>
    );
  }

  if (permission === VIEWER) {
    return (
      <Wrapper>
        <Alert
          variant="outlined"
          severity="info"
          sx={{ width: '100%' }}
          action={
            <Button
              variant="outlined"
              size="small"
              disableElevation
              onClick={() => duplicateFileAction.run({ name, submit })}
            >
              {duplicateFileAction.label}
            </Button>
          }
        >
          <strong>Read-only.</strong> To edit this file, make a duplicate in your files.
        </Alert>
      </Wrapper>
    );
  }

  return null;
}

function Wrapper({ children }: { children: React.ReactNode }) {
  const theme = useTheme();
  return (
    <Paper
      sx={{
        position: 'fixed',
        bottom: theme.spacing(8),
        transform: 'translateX(-50%)',
        left: '50%',
        width: '95%',
        maxWidth: '40rem',
        zIndex: '10',
      }}
      elevation={4}
    >
      {children}
    </Paper>
  );
}
