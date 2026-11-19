import type { CSSProperties } from 'react';
import { useGame } from '../../context/GameProvider';

const DIGITS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

interface MicroSpark {
  tx: string;
  ty: string;
  rot: string;
  color: string;
  delay: string;
}

const MICRO_SPARKS: readonly MicroSpark[] = [
  { tx: '-26px', ty: '-22px', rot: '-45deg', color: '#0066cc', delay: '0ms' },
  { tx: '-10px', ty: '-30px', rot: '25deg', color: '#34c759', delay: '20ms' },
  { tx: '12px', ty: '-28px', rot: '-20deg', color: '#f5b942', delay: '10ms' },
  { tx: '28px', ty: '-18px', rot: '50deg', color: '#5e9eff', delay: '25ms' },
  { tx: '30px', ty: '4px', rot: '80deg', color: '#af52de', delay: '15ms' },
  { tx: '22px', ty: '24px', rot: '120deg', color: '#34c759', delay: '30ms' },
  { tx: '-2px', ty: '28px', rot: '-75deg', color: '#f5b942', delay: '10ms' },
  { tx: '-24px', ty: '22px', rot: '-110deg', color: '#ff6961', delay: '20ms' },
  { tx: '-30px', ty: '2px', rot: '-55deg', color: '#5e9eff', delay: '5ms' },
  { tx: '0px', ty: '-34px', rot: '40deg', color: '#0066cc', delay: '35ms' },
];

/**
 * 3x3 digit input pad showing remaining counts, subtly dimming completed digits,
 * and firing a crisp micro-confetti burst the moment a digit reaches all 9 placements.
 */
export function NumberPad() {
  const { state, actions, meta } = useGame();
  const { game, selectedCell, notesMode, celebratingDigit } = state;

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
        const isCelebrating = celebratingDigit === digit;

        return (
          <button
            key={digit}
            type="button"
            disabled={disabled}
            data-celebrating={isCelebrating ? 'true' : undefined}
            onClick={() => void actions.enterDigit(digit)}
            aria-label={`Enter ${digit}${notesMode ? ' as Note' : ''}, ${
              completed ? 'all 9 placed' : `${remaining} remaining`
            }`}
            className={`relative flex h-12 flex-col items-center justify-center overflow-visible rounded-lg border transition-colors disabled:cursor-not-allowed ${
              isCelebrating
                ? 'border-[var(--ink-success)] bg-[var(--cell-hint)] text-[var(--ink-success)] ring-2 ring-[var(--ink-success)]/40'
                : completed
                  ? 'border-[var(--border-hairline)] bg-[var(--surface-subtle)]/60 text-[var(--ink-muted)] opacity-45'
                  : 'border-[var(--border-hairline)] bg-[var(--surface-elevated)] text-[var(--ink-primary)] hover:border-[var(--accent)] hover:bg-[var(--accent-soft)]/50'
            }`}
          >
            <span className="tabular-nums text-[19px] font-semibold leading-none">{digit}</span>
            <span className="tabular-nums mt-0.5 text-[10px] leading-none text-[var(--ink-secondary)]">
              {completed ? 'Done' : remaining}
            </span>

            {isCelebrating ? (
              <span
                aria-hidden="true"
                data-testid={`digit-confetti-${digit}`}
                className="pointer-events-none absolute inset-0 overflow-visible"
              >
                {MICRO_SPARKS.map((spark, idx) => (
                  <span
                    key={idx}
                    className="animate-micro-confetti absolute left-1/2 top-1/2 h-1.5 w-2 rounded-[1px]"
                    style={
                      {
                        backgroundColor: spark.color,
                        animationDelay: spark.delay,
                        '--tx': spark.tx,
                        '--ty': spark.ty,
                        '--rot': spark.rot,
                      } as CSSProperties
                    }
                  />
                ))}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
