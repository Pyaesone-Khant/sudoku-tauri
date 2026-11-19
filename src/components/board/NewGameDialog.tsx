import { X } from 'lucide-react';
import { DIFFICULTIES, type Difficulty } from '../../api/types';

interface NewGameDialogProps {
  open: boolean;
  hasInProgressStandard: boolean;
  isBusy: boolean;
  onSelectDifficulty: (difficulty: Difficulty) => void;
  onClose: () => void;
}

const DIFFICULTY_DESCRIPTIONS: Record<Difficulty, string> = {
  Easy: 'Scanning & singles · Target 5 min',
  Medium: 'Locked candidates & pairs · Target 10 min',
  Hard: 'Triples & X-Wing · Target 18 min',
  Expert: 'XY-Wing, Swordfish & chains · Target 30 min',
};

/**
 * Modal dialog to start a new Standard Game with explicit abandonment warning when needed.
 */
export function NewGameDialog({
  open,
  hasInProgressStandard,
  isBusy,
  onSelectDifficulty,
  onClose,
}: NewGameDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="New Standard Game"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-sm rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-5 shadow-xl">
        <div className="flex items-center justify-between">
          <h2 className="text-[16px] font-semibold text-[var(--ink-primary)]">
            New Standard Game
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close dialog"
            className="flex h-7 w-7 items-center justify-center rounded-md text-[var(--ink-secondary)] hover:bg-[var(--surface-subtle)]"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {hasInProgressStandard ? (
          <p className="mt-2 rounded-lg bg-[var(--cell-mistake-bg)] px-3 py-2 text-[12px] text-[var(--ink-mistake)]">
            Starting a new Standard Game will abandon your current in-progress Standard Game and reset your Win Streak. Your Daily Challenge slot stays untouched.
          </p>
        ) : (
          <p className="mt-1.5 text-[12px] text-[var(--ink-secondary)]">
            Choose a Difficulty graded by human solving technique.
          </p>
        )}

        <div className="mt-4 flex flex-col gap-2">
          {DIFFICULTIES.map((difficulty) => (
            <button
              key={difficulty}
              type="button"
              disabled={isBusy}
              onClick={() => onSelectDifficulty(difficulty)}
              className="flex items-center justify-between rounded-xl border border-[var(--border-hairline)] bg-[var(--surface-canvas)] px-3.5 py-2.5 text-left transition-colors hover:border-[var(--accent)] hover:bg-[var(--accent-soft)]/40 disabled:opacity-50"
            >
              <div>
                <div className="text-[13px] font-semibold text-[var(--ink-primary)]">
                  {difficulty}
                </div>
                <div className="text-[11px] text-[var(--ink-secondary)]">
                  {DIFFICULTY_DESCRIPTIONS[difficulty]}
                </div>
              </div>
              <span className="text-[12px] font-medium text-[var(--accent)]">Start</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
