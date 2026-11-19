import { Lightbulb, X } from 'lucide-react';
import { formatTechniqueName } from '../../api/types';
import { useGame } from '../../context/GameProvider';

/**
 * Displays the active Smart Hint explanation with Apply and Dismiss actions.
 */
export function HintBanner() {
  const { state, actions, meta } = useGame();
  const hint = state.game?.active_hint;

  if (!hint || state.game?.status !== 'Active') {
    return null;
  }

  const title =
    hint.kind === 'FixMistake'
      ? 'Fix Mistake first'
      : formatTechniqueName(hint.technique);
  const ctaLabel = hint.kind === 'FixMistake' ? 'Clear Mistake' : `Apply ${hint.digit}`;

  return (
    <div
      role="status"
      aria-live="polite"
      className="flex items-start justify-between gap-3 rounded-xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] px-3.5 py-2.5 shadow-xs"
    >
      <div className="flex items-start gap-2.5">
        <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-[var(--cell-hint)] text-[var(--ink-success)]">
          <Lightbulb className="h-3.5 w-3.5" />
        </span>
        <div>
          <p className="text-[12px] font-semibold text-[var(--ink-primary)]">{title}</p>
          <p className="mt-0.5 text-[12px] leading-snug text-[var(--ink-secondary)]">
            {hint.explanation}
          </p>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        <button
          type="button"
          disabled={meta.isBusy}
          onClick={() => void actions.applyHint()}
          className="rounded-md bg-[var(--accent)] px-2.5 py-1 text-[12px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          {ctaLabel}
        </button>
        <button
          type="button"
          onClick={() => void actions.dismissHint()}
          aria-label="Dismiss Hint"
          className="flex h-6 w-6 items-center justify-center rounded-md text-[var(--ink-secondary)] hover:bg-[var(--surface-subtle)] hover:text-[var(--ink-primary)]"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}
