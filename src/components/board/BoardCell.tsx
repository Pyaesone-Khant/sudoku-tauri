import { memo } from 'react';
import type { CellSnapshot } from '../../api/types';

interface BoardCellProps {
  cell: CellSnapshot;
  isSelected: boolean;
  isPeer: boolean;
  isSameDigit: boolean;
  isHintTarget: boolean;
  isHintHighlight: boolean;
  isCelebrating: boolean;
  isObscured: boolean;
  onSelect: (index: number) => void;
}

const NOTE_DIGITS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

/**
 * Memoized single Sudoku cell rendering Given, player entry, Mistake, Hint, and 3x3 Notes.
 */
export const BoardCell = memo(function BoardCell({
  cell,
  isSelected,
  isPeer,
  isSameDigit,
  isHintTarget,
  isHintHighlight,
  isCelebrating,
  isObscured,
  onSelect,
}: BoardCellProps) {
  const row = Math.floor(cell.index / 9);
  const col = cell.index % 9;

  const borderClasses = [
    col % 3 === 2 && col !== 8 ? 'border-r-[2px] border-r-[var(--border-strong)]' : 'border-r border-r-[var(--border-hairline)]',
    row % 3 === 2 && row !== 8 ? 'border-b-[2px] border-b-[var(--border-strong)]' : 'border-b border-b-[var(--border-hairline)]',
    col === 8 ? 'border-r-0' : '',
    row === 8 ? 'border-b-0' : '',
  ].join(' ');

  let bgClass = 'bg-[var(--surface-elevated)]';
  if (!isObscured) {
    if (cell.is_mistake) {
      bgClass = isSelected ? 'bg-[var(--cell-mistake-bg)] ring-2 ring-inset ring-[var(--ink-mistake)]' : 'bg-[var(--cell-mistake-bg)]';
    } else if (isSelected) {
      bgClass = 'bg-[var(--cell-selected)]';
    } else if (isHintTarget) {
      bgClass = 'bg-[var(--cell-hint)] ring-2 ring-inset ring-[var(--ink-success)]';
    } else if (isHintHighlight) {
      bgClass = 'bg-[var(--cell-hint)]/70';
    } else if (isSameDigit) {
      bgClass = 'bg-[var(--cell-same-digit)]';
    } else if (isPeer) {
      bgClass = 'bg-[var(--cell-peer)]';
    }
  }

  let inkClass = 'text-[var(--accent)] font-medium';
  if (cell.is_mistake) {
    inkClass = 'text-[var(--ink-mistake)] font-semibold';
  } else if (cell.is_given) {
    inkClass = 'text-[var(--ink-primary)] font-semibold';
  } else if (cell.is_hinted) {
    inkClass = 'text-[var(--ink-success)] font-medium';
  }

  const celebrationDelayMs = isCelebrating ? (row + col) * 22 : 0;

  return (
    <button
      type="button"
      onClick={() => onSelect(cell.index)}
      style={isCelebrating ? { animationDelay: `${celebrationDelayMs}ms` } : undefined}
      aria-label={`Row ${row + 1}, Column ${col + 1}${
        isObscured
          ? ', hidden while paused'
          : cell.value !== null
            ? `, ${cell.value}${cell.is_given ? ', Given' : cell.is_mistake ? ', Mistake' : ''}`
            : ', empty'
      }`}
      aria-pressed={isSelected}
      className={`relative flex aspect-square items-center justify-center p-0.5 transition-colors duration-75 focus:outline-none ${borderClasses} ${bgClass} ${
        isCelebrating ? 'animate-cell-complete z-10' : ''
      }`}
    >
      {isObscured ? null : cell.value !== null ? (
        <span className={`tabular-nums text-[clamp(18px,3.2vh,25px)] leading-none ${inkClass}`}>
          {cell.value}
        </span>
      ) : cell.notes.length > 0 ? (
        <span className="grid h-full w-full grid-cols-3 grid-rows-3 place-items-center">
          {NOTE_DIGITS.map((d) => (
            <span
              key={d}
              className="tabular-nums text-[clamp(8px,1.25vh,10px)] font-medium leading-none text-[var(--ink-secondary)]"
            >
              {cell.notes.includes(d) ? d : ''}
            </span>
          ))}
        </span>
      ) : null}
    </button>
  );
});
