import { useGame } from '../../context/GameProvider';

const DIGITS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

/**
 * 3x3 digit input pad showing remaining counts and subtly dimming completed digits.
 */
export function NumberPad() {
  const { state, actions, meta } = useGame();
  const { game, selectedCell, notesMode } = state;

  if (!game) {
    return null;
  }

  const disabled = game.status !== 'Active' || selectedCell === null || meta.isBusy;

  return (
    <div className="grid grid-cols-3 gap-2" aria-label="Number pad">
      {DIGITS.map((digit) => {
        const placed = game.correct_digit_counts[digit - 1] ?? 0;
        const remaining = Math.max(0, 9 - placed);
        const completed = remaining === 0;

        return (
          <button
            key={digit}
            type="button"
            disabled={disabled}
            onClick={() => void actions.enterDigit(digit)}
            aria-label={`Enter ${digit}${notesMode ? ' as Note' : ''}, ${remaining} remaining`}
            className={`relative flex h-12 flex-col items-center justify-center rounded-lg border transition-colors disabled:cursor-not-allowed ${
              completed
                ? 'border-[var(--border-hairline)] bg-[var(--surface-subtle)]/60 text-[var(--ink-muted)] opacity-45'
                : 'border-[var(--border-hairline)] bg-[var(--surface-elevated)] text-[var(--ink-primary)] hover:border-[var(--accent)] hover:bg-[var(--accent-soft)]/50'
            }`}
          >
            <span className="tabular-nums text-[19px] font-semibold leading-none">{digit}</span>
            <span className="tabular-nums mt-0.5 text-[10px] leading-none text-[var(--ink-secondary)]">
              {completed ? 'Done' : remaining}
            </span>
          </button>
        );
      })}
    </div>
  );
}
