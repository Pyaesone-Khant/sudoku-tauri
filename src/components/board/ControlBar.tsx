import { Eraser, Lightbulb, Pause, Pencil, Play, Redo2, Undo2 } from 'lucide-react';
import { useGame } from '../../context/GameProvider';

/**
 * Primary solving tools: Undo, Redo, Erase, Notes mode toggle, Smart Hint, and Pause/Resume.
 */
export function ControlBar() {
  const { state, actions, meta } = useGame();
  const { game, notesMode } = state;

  if (!game) {
    return null;
  }

  const isActive = game.status === 'Active';
  const isPaused = game.status === 'Paused';
  const disabledMove = !isActive || meta.isBusy;

  const baseBtn =
    'flex flex-col items-center justify-center gap-1 rounded-lg border border-[var(--border-hairline)] bg-[var(--surface-elevated)] py-2 text-[11px] font-medium text-[var(--ink-primary)] transition-colors hover:bg-[var(--surface-subtle)] disabled:cursor-not-allowed disabled:opacity-40';

  return (
    <div className="grid grid-cols-3 gap-2">
      <button
        type="button"
        disabled={disabledMove || !game.can_undo}
        onClick={() => void actions.undo()}
        title="Undo (⌘Z)"
        className={baseBtn}
      >
        <Undo2 className="h-4 w-4" />
        Undo
      </button>

      <button
        type="button"
        disabled={disabledMove || !game.can_redo}
        onClick={() => void actions.redo()}
        title="Redo (⌘⇧Z)"
        className={baseBtn}
      >
        <Redo2 className="h-4 w-4" />
        Redo
      </button>

      <button
        type="button"
        disabled={disabledMove || state.selectedCell === null}
        onClick={() => void actions.eraseSelected()}
        title="Erase (⌫)"
        className={baseBtn}
      >
        <Eraser className="h-4 w-4" />
        Erase
      </button>

      <button
        type="button"
        disabled={!isActive}
        aria-pressed={notesMode}
        onClick={actions.toggleNotesMode}
        title="Toggle Notes (N)"
        className={`flex flex-col items-center justify-center gap-1 rounded-lg border py-2 text-[11px] font-medium transition-colors disabled:opacity-40 ${
          notesMode
            ? 'border-[var(--accent)] bg-[var(--accent-soft)] text-[var(--accent)]'
            : 'border-[var(--border-hairline)] bg-[var(--surface-elevated)] text-[var(--ink-primary)] hover:bg-[var(--surface-subtle)]'
        }`}
      >
        <Pencil className="h-4 w-4" />
        Notes
      </button>

      <button
        type="button"
        disabled={disabledMove}
        onClick={() => void actions.requestHint()}
        title="Smart Hint (H)"
        className={baseBtn}
      >
        <Lightbulb className="h-4 w-4" />
        Hint
      </button>

      <button
        type="button"
        disabled={game.status === 'Completed' || meta.isBusy}
        onClick={() => void (isPaused ? actions.resumeGame() : actions.pauseGame())}
        title="Pause / Resume (P)"
        className={baseBtn}
      >
        {isPaused ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
        {isPaused ? 'Resume' : 'Pause'}
      </button>
    </div>
  );
}
