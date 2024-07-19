import { events } from '@/app/events/events';
import { pluralize } from '@/app/helpers/pluralize';
import { multiplayer } from '@/app/web-workers/multiplayerWebWorker/multiplayer';
import { MultiplayerState } from '@/app/web-workers/multiplayerWebWorker/multiplayerClientMessages';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/shared/shadcn/ui/dropdown-menu';
import { Check, ErrorOutline } from '@mui/icons-material';
import { CircularProgress, Tooltip, useTheme } from '@mui/material';
import { useEffect, useState } from 'react';
import BottomBarItem from './BottomBarItem';

export default function SyncState() {
  const theme = useTheme();

  const [syncState, setSyncState] = useState<MultiplayerState>(multiplayer.state);

  useEffect(() => {
    const updateState = (state: MultiplayerState) => setSyncState(state);
    events.on('multiplayerState', updateState);
    return () => {
      events.off('multiplayerState', updateState);
    };
  }, []);

  const [unsavedTransactions, setUnsavedTransactions] = useState(0);
  useEffect(() => {
    const updateUnsavedTransactions = (transactions: number, _operations: number) => {
      setUnsavedTransactions(transactions);
    };
    events.on('offlineTransactions', updateUnsavedTransactions);
    return () => {
      events.off('offlineTransactions', updateUnsavedTransactions);
    };
  }, []);

  const [open, setOpen] = useState(false);

  let tooltip: string;
  let icon: JSX.Element;
  let message: string | JSX.Element;

  if (['waiting to reconnect', 'connecting'].includes(syncState) && multiplayer.brokenConnection) {
    icon = <CircularProgress size="0.5rem" />;
    message = <span style={{ color: theme.palette.error.main }}>Reconnecting…</span>;
    tooltip =
      'Attempting to connect to the Quadratic server after losing connection. Your changes may only be saved locally…';
  } else if (['not connected', 'connecting', 'waiting to reconnect', 'startup'].includes(syncState)) {
    icon = <CircularProgress size="0.5rem" />;
    message = <span>Connecting…</span>;
    tooltip = 'Connecting to the Quadratic server…';
  } else if (syncState === 'syncing') {
    icon = <CircularProgress size="0.5rem" />;
    message = <span>Syncing...</span>;
    tooltip = 'Syncing changes to the Quadratic server. Your recent changes are saved locally.';
  } else if (syncState === 'connected') {
    icon = <Check fontSize="inherit" />;
    message = <span>Connected</span>;
    tooltip = 'Connected to the Quadratic server. Your changes are saved.';
  } else if (syncState === 'no internet') {
    icon = <ErrorOutline fontSize="inherit" style={{ color: theme.palette.error.main }} />;
    message = <span style={{ color: theme.palette.error.main }}>Offline</span>;
    tooltip = 'Your internet connection appears not to be working. Your changes are only saved locally.';
  } else {
    icon = <ErrorOutline fontSize="inherit" style={{ color: theme.palette.error.main }} />;
    tooltip =
      "Connection to the Quadratic server was lost. We'll continue trying to reconnect. Your changes are only saved locally.";
    message = <span style={{ color: theme.palette.error.main }}>Offline</span>;
  }

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <BottomBarItem icon={icon} onClick={() => {}}>
          <Tooltip title={tooltip}>{message}</Tooltip>
        </BottomBarItem>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuLabel className={`flexz zw-full zjustify-between`}>Status</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem className={`flexz zw-full zjustify-between`}>
          {unsavedTransactions === 0
            ? 'Nothing waiting to sync'
            : `Syncing ${unsavedTransactions} ${pluralize('item', unsavedTransactions)}.`}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
