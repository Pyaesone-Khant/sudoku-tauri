import { useMemo } from 'react';
import { Play } from 'lucide-react';
import { useGame } from '../../context/GameProvider';
import { BoardCell } from './BoardCell';

/**
 * 9x9 Sudoku board with crisp 3x3 box boundaries, smart highlights, and frosted Pause veil.
 */
export function SudokuBoard() {
  const { state, actions } = useGame();
  const { game, selectedCell } = state;

  const hintHighlightSet = useMemo(() => {
    const set = new Set<number>();
    const hint = game?.active_hint;
    if (hint?.kind === 'PlaceDigit') {
      for (const idx of hint.highlight_cells) {
        set.add(idx);
      }
    }
    return set;
  }, [game?.active_hint]);

  if (!game) {
    return null;
  }

  const isPaused = game.status === 'Paused';
  const selectedRow = selectedCell !== null ? Math.floor(selectedCell / 9) : -1;
  const selectedCol = selectedCell !== null ? selectedCell % 9 : -1;
  const selectedBox =
    selectedCell !== null ? Math.floor(selectedRow / 3) * 3 + Math.floor(selectedCol / 3) : -1;
  const selectedDigit =
    selectedCell !== null ? (game.cells[selectedCell]?.value ?? null) : null;
  const hintTargetCell = game.active_hint ? game.active_hint.cell : -1;

  return (
    <div className="relative aspect-square w-[min(460px,calc(100vh-146px))] max-w-full shrink-0 overflow-hidden rounded-xl border-2 border-[var(--border-strong)] bg-[var(--surface-elevated)] shadow-xs">
      <div
        role="grid"
        aria-label="Sudoku board"
        className="grid h-full w-full grid-cols-9 grid-rows-9"
      >
        {game.cells.map((cell) => {
          const r = Math.floor(cell.index / 9);
          const c = cell.index % 9;
          const b = Math.floor(r / 3) * 3 + Math.floor(c / 3);
          const isSelected = cell.index === selectedCell;
          const isPeer =
            !isSelected &&
            selectedCell !== null &&
            (r === selectedRow || c === selectedCol || b === selectedBox);
          const isSameDigit =
            !isSelected &&
            selectedDigit !== null &&
            cell.value === selectedDigit;

          return (
            <BoardCell
              key={cell.index}
              cell={cell}
              isSelected={isSelected}
              isPeer={isPeer}
              isSameDigit={isSameDigit}
              isHintTarget={cell.index === hintTargetCell}
              isHintHighlight={hintHighlightSet.has(cell.index)}
              isObscured={isPaused}
              onSelect={actions.selectCell}
            />
          );
        })}
      </div>

      {isPaused ? (
        <div
          role="dialog"
          aria-label="Game paused"
          className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-[var(--surface-elevated)]/92 backdrop-blur-md"
        >
          <p className="text-[15px] font-semibold text-[var(--ink-primary)]">Paused</p>
          <p className="text-[12px] text-[var(--ink-secondary)]">
            Board hidden while Solve Time is paused.
          </p>
          <button
            type="button"
            onClick={() => void actions.resumeGame()}
            className="mt-1 inline-flex items-center gap-1.5 rounded-lg bg-[var(--accent)] px-4 py-2 text-[13px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)]"
          >
            <Play className="h-3.5 w-3.5 fill-current" />
            Resume
          </button>
        </div>
      ) : null}
    </div>
  );
}
