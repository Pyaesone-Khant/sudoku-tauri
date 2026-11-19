import type { SudokuClient } from './api/sudokuClient';
import { PlayView } from './components/board/PlayView';
import { DailyCalendarView } from './components/daily/DailyCalendarView';
import { TitleBar } from './components/layout/TitleBar';
import { StatsView } from './components/stats/StatsView';
import { GameProvider, useGame } from './context/GameProvider';
import { PreferencesProvider } from './context/PreferencesProvider';

function AppShell() {
  const { state, actions } = useGame();

  return (
    <div className="flex h-full flex-col bg-[var(--surface-canvas)] text-[var(--ink-primary)]">
      <TitleBar />

      {state.errorMessage ? (
        <div
          role="alert"
          className="flex items-center justify-between border-b border-[var(--border-hairline)] bg-[var(--cell-mistake-bg)] px-4 py-2 text-[12px] text-[var(--ink-mistake)]"
        >
          <span>{state.errorMessage}</span>
          <button
            type="button"
            onClick={actions.clearError}
            className="font-medium underline hover:opacity-80"
          >
            Dismiss
          </button>
        </div>
      ) : null}

      <main className="flex flex-1 flex-col overflow-y-auto">
        {state.activeTab === 'play' ? (
          <PlayView />
        ) : state.activeTab === 'daily' ? (
          <DailyCalendarView />
        ) : (
          <StatsView />
        )}
      </main>
    </div>
  );
}

interface AppProps {
  /** Optional injected IPC client for isolated component tests. */
  client?: SudokuClient;
}

/**
 * Root Sudoku desktop application wrapping appearance preferences and game state.
 */
export function App({ client }: AppProps) {
  return (
    <PreferencesProvider>
      <GameProvider client={client}>
        <AppShell />
      </GameProvider>
    </PreferencesProvider>
  );
}
