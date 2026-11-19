import { Award, Calendar, Plus } from 'lucide-react';
import { formatDisplayDate, formatSolveTime, type GameSnapshot } from '../../api/types';

interface VictorySummaryProps {
  game: GameSnapshot;
  onNewStandardGame: () => void;
  onOpenDailyCalendar: () => void;
}

/**
 * Displays the completed Game score breakdown: Earned Score, Time Bonus, Penalty, and Final Score.
 */
export function VictorySummary({
  game,
  onNewStandardGame,
  onOpenDailyCalendar,
}: VictorySummaryProps) {
  if (game.status !== 'Completed') {
    return null;
  }

  const subtitle =
    game.kind.type === 'Daily'
      ? `Daily Challenge · ${formatDisplayDate(game.kind.date)} · ${game.difficulty}`
      : `Standard Game · ${game.difficulty}`;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Puzzle completed"
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/35 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-sm rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-6 shadow-xl">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-[20px] font-semibold tracking-tight text-[var(--ink-primary)]">
              Puzzle Solved
            </h2>
            <p className="mt-0.5 text-[12px] text-[var(--ink-secondary)]">{subtitle}</p>
          </div>
          {game.flawless ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-[var(--cell-hint)] px-2.5 py-1 text-[11px] font-semibold text-[var(--ink-success)]">
              <Award className="h-3.5 w-3.5" />
              Flawless Solve
            </span>
          ) : null}
        </div>

        <div className="mt-5 rounded-xl bg-[var(--surface-canvas)] p-4">
          <div className="flex items-baseline justify-between">
            <span className="text-[12px] font-medium text-[var(--ink-secondary)]">Final Score</span>
            <span className="tabular-nums text-[28px] font-bold tracking-tight text-[var(--ink-primary)]">
              {game.score.final_score.toLocaleString()}
            </span>
          </div>

          <div className="mt-3 space-y-1.5 border-t border-[var(--border-hairline)] pt-3 text-[12px]">
            <div className="flex justify-between">
              <span className="text-[var(--ink-secondary)]">Earned Score</span>
              <span className="tabular-nums font-medium text-[var(--ink-primary)]">
                +{game.score.earned_score.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--ink-secondary)]">
                Time Bonus ({formatSolveTime(game.solve_time_secs)})
              </span>
              <span className="tabular-nums font-medium text-[var(--ink-success)]">
                +{game.score.time_bonus.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--ink-secondary)]">
                Mistake Penalty ({game.mistakes})
              </span>
              <span className="tabular-nums font-medium text-[var(--ink-mistake)]">
                −{game.score.mistake_penalty.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--ink-secondary)]">
                Hint Penalty ({game.hints_used})
              </span>
              <span className="tabular-nums font-medium text-[var(--ink-mistake)]">
                −{game.score.hint_penalty.toLocaleString()}
              </span>
            </div>
          </div>
        </div>

        <div className="mt-5 grid grid-cols-2 gap-2.5">
          <button
            type="button"
            onClick={onNewStandardGame}
            className="inline-flex items-center justify-center gap-1.5 rounded-lg bg-[var(--accent)] px-3 py-2 text-[13px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)]"
          >
            <Plus className="h-4 w-4" />
            New Game
          </button>
          <button
            type="button"
            onClick={onOpenDailyCalendar}
            className="inline-flex items-center justify-center gap-1.5 rounded-lg border border-[var(--border-hairline)] bg-[var(--surface-canvas)] px-3 py-2 text-[13px] font-medium text-[var(--ink-primary)] transition-colors hover:bg-[var(--surface-subtle)]"
          >
            <Calendar className="h-4 w-4" />
            Daily Calendar
          </button>
        </div>
      </div>
    </div>
  );
}
