import { useState } from 'react';
import { Calendar, Plus } from 'lucide-react';
import {
  DIFFICULTIES,
  formatDisplayDate,
  formatIsoDate,
  formatSolveTime,
  type Difficulty,
} from '../../api/types';
import { useGame } from '../../context/GameProvider';
import { useKeyboardControls } from '../../hooks/useKeyboardControls';
import { ControlBar } from './ControlBar';
import { HintBanner } from './HintBanner';
import { NewGameDialog } from './NewGameDialog';
import { NumberPad } from './NumberPad';
import { SudokuBoard } from './SudokuBoard';
import { VictorySummary } from './VictorySummary';

/**
 * Primary solving workspace pairing the 9x9 Sudoku board with live metrics and controls.
 */
export function PlayView() {
  const { state, actions, meta } = useGame();
  const { appState, game, displaySolveTimeSecs } = state;
  const [newGameOpen, setNewGameOpen] = useState(false);

  useKeyboardControls(state.activeTab === 'play' && !newGameOpen);

  const handleStartStandard = async (difficulty: Difficulty) => {
    setNewGameOpen(false);
    await actions.startStandardGame(difficulty);
  };

  if (meta.isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center text-[13px] text-[var(--ink-secondary)]">
        Loading…
      </div>
    );
  }

  // Empty welcome state before the player starts their first Game.
  if (!game) {
    const todayIso = appState ? formatIsoDate(appState.today) : null;
    return (
      <section className="mx-auto flex w-full max-w-md flex-1 flex-col items-center justify-center px-6 py-8 text-center">
        <h1 className="text-[24px] font-semibold tracking-tight text-[var(--ink-primary)]">
          Start a Puzzle
        </h1>
        <p className="mt-1.5 text-[13px] text-[var(--ink-secondary)]">
          Every Puzzle has a unique solution graded by human solving technique.
        </p>

        <div className="mt-6 grid w-full grid-cols-2 gap-2.5">
          {DIFFICULTIES.map((difficulty) => (
            <button
              key={difficulty}
              type="button"
              disabled={meta.isBusy}
              onClick={() => void actions.startStandardGame(difficulty)}
              className="rounded-xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] px-4 py-3 text-left transition-colors hover:border-[var(--accent)] hover:bg-[var(--accent-soft)]/40 disabled:opacity-50"
            >
              <div className="text-[14px] font-semibold text-[var(--ink-primary)]">
                {difficulty}
              </div>
              <div className="mt-0.5 text-[11px] text-[var(--ink-secondary)]">Standard Game</div>
            </button>
          ))}
        </div>

        {todayIso ? (
          <button
            type="button"
            disabled={meta.isBusy}
            onClick={() => void actions.openDailyChallenge(todayIso)}
            className="mt-4 inline-flex w-full items-center justify-center gap-2 rounded-xl bg-[var(--accent)] px-4 py-2.5 text-[13px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            <Calendar className="h-4 w-4" />
            Play Today&apos;s Daily Challenge
          </button>
        ) : null}
      </section>
    );
  }

  const hasBothSlots = Boolean(
    appState?.has_in_progress_standard && appState?.in_progress_daily_date,
  );
  const modeLabel =
    game.kind.type === 'Daily'
      ? `Daily · ${formatDisplayDate(game.kind.date)}`
      : 'Standard Game';

  return (
    <section className="mx-auto flex w-full max-w-[820px] flex-1 flex-col justify-center gap-3 px-5 py-4">
      {game.active_hint ? <HintBanner /> : null}

      <div className="flex flex-wrap items-start justify-center gap-6">
        <SudokuBoard />

        <aside className="flex w-[246px] shrink-0 flex-col gap-3.5">
          {/* Header & optional dual-slot switcher */}
          <div className="flex items-start justify-between gap-2">
            <div>
              <div className="text-[16px] font-semibold tracking-tight text-[var(--ink-primary)]">
                {game.difficulty}
              </div>
              <div className="text-[12px] text-[var(--ink-secondary)]">{modeLabel}</div>
            </div>

            {hasBothSlots ? (
              <button
                type="button"
                onClick={() =>
                  void actions.switchActiveSlot(
                    appState?.active_slot === 'Standard' ? 'Daily' : 'Standard',
                  )
                }
                className="rounded-md border border-[var(--border-hairline)] bg-[var(--surface-elevated)] px-2 py-1 text-[11px] font-medium text-[var(--accent)] hover:bg-[var(--accent-soft)]/50"
              >
                {appState?.active_slot === 'Standard' ? 'Resume Daily' : 'Resume Standard'}
              </button>
            ) : null}
          </div>

          {/* Live Solve Time, Mistakes, and Score */}
          <div className="grid grid-cols-3 divide-x divide-[var(--border-hairline)] rounded-xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] py-2.5 text-center">
            <div className="px-1.5">
              <div className="text-[10px] font-medium text-[var(--ink-secondary)]">Solve Time</div>
              <div className="tabular-nums mt-0.5 text-[14px] font-semibold text-[var(--ink-primary)]">
                {formatSolveTime(displaySolveTimeSecs)}
              </div>
            </div>
            <div className="px-1.5">
              <div className="text-[10px] font-medium text-[var(--ink-secondary)]">Mistakes</div>
              <div
                className={`tabular-nums mt-0.5 text-[14px] font-semibold ${
                  game.mistakes > 0 ? 'text-[var(--ink-mistake)]' : 'text-[var(--ink-primary)]'
                }`}
              >
                {game.mistakes}
              </div>
            </div>
            <div className="px-1.5">
              <div className="text-[10px] font-medium text-[var(--ink-secondary)]">Score</div>
              <div className="tabular-nums mt-0.5 text-[14px] font-semibold text-[var(--ink-primary)]">
                {game.score.final_score.toLocaleString()}
              </div>
            </div>
          </div>

          <ControlBar />
          <NumberPad />

          <button
            type="button"
            onClick={() => setNewGameOpen(true)}
            className="inline-flex items-center justify-center gap-1.5 rounded-lg border border-[var(--border-hairline)] bg-[var(--surface-elevated)] py-2 text-[12px] font-medium text-[var(--ink-primary)] transition-colors hover:bg-[var(--surface-subtle)]"
          >
            <Plus className="h-3.5 w-3.5" />
            New Standard Game
          </button>
        </aside>
      </div>

      <NewGameDialog
        open={newGameOpen}
        hasInProgressStandard={Boolean(appState?.has_in_progress_standard)}
        isBusy={meta.isBusy}
        onSelectDifficulty={(d) => void handleStartStandard(d)}
        onClose={() => setNewGameOpen(false)}
      />

      <VictorySummary
        game={game}
        onNewStandardGame={() => setNewGameOpen(true)}
        onOpenDailyCalendar={() => actions.setActiveTab('daily')}
      />
    </section>
  );
}
