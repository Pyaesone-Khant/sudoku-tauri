import { useEffect } from 'react';
import { useGame } from '../context/GameProvider';

/**
 * Registers macOS keyboard shortcuts when the Play screen is active.
 * Supports Arrow keys / WASD navigation, 1–9 entry, Shift+1–9 / N for Notes,
 * Backspace/Delete to erase, ⌘Z/⌘⇧Z for Undo/Redo, H for Hint, and P for Pause/Resume.
 */
export function useKeyboardControls(enabled: boolean): void {
  const { state, actions } = useGame();
  const { game, selectedCell } = state;

  useEffect(() => {
    if (!enabled || !game) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')) {
        return;
      }

      const key = event.key;
      const isMod = event.metaKey || event.ctrlKey;

      // Pause / Resume toggle with P
      if (!isMod && (key === 'p' || key === 'P')) {
        event.preventDefault();
        if (game?.status === 'Active') {
          void actions.pauseGame();
        } else if (game?.status === 'Paused') {
          void actions.resumeGame();
        }
        return;
      }

      if (game?.status !== 'Active') {
        return;
      }

      // Undo (⌘Z) / Redo (⌘⇧Z or ⌘Y)
      if (isMod && (key === 'z' || key === 'Z')) {
        event.preventDefault();
        if (event.shiftKey) {
          void actions.redo();
        } else {
          void actions.undo();
        }
        return;
      }
      if (isMod && (key === 'y' || key === 'Y')) {
        event.preventDefault();
        void actions.redo();
        return;
      }

      if (isMod || event.altKey) {
        return;
      }

      // Grid navigation (Arrow keys and WASD)
      const moveDelta = (dir: 'up' | 'down' | 'left' | 'right') => {
        const current = selectedCell ?? 40;
        const row = Math.floor(current / 9);
        const col = current % 9;
        const nextRow = dir === 'up' ? (row + 8) % 9 : dir === 'down' ? (row + 1) % 9 : row;
        const nextCol = dir === 'left' ? (col + 8) % 9 : dir === 'right' ? (col + 1) % 9 : col;
        actions.selectCell(nextRow * 9 + nextCol);
      };

      if (key === 'ArrowUp' || key === 'w' || key === 'W') {
        event.preventDefault();
        moveDelta('up');
        return;
      }
      if (key === 'ArrowDown' || key === 's' || key === 'S') {
        event.preventDefault();
        moveDelta('down');
        return;
      }
      if (key === 'ArrowLeft' || key === 'a' || key === 'A') {
        event.preventDefault();
        moveDelta('left');
        return;
      }
      if (key === 'ArrowRight' || key === 'd' || key === 'D') {
        event.preventDefault();
        moveDelta('right');
        return;
      }

      // Notes mode toggle (N)
      if (key === 'n' || key === 'N') {
        event.preventDefault();
        actions.toggleNotesMode();
        return;
      }

      // Smart Hint (H)
      if (key === 'h' || key === 'H') {
        event.preventDefault();
        void actions.requestHint();
        return;
      }

      // Erase (Backspace / Delete / 0)
      if (key === 'Backspace' || key === 'Delete' || key === '0') {
        event.preventDefault();
        void actions.eraseSelected();
        return;
      }

      // Digit entry (1–9, plus Shift+1–9 via event.code for quick Notes)
      const digitFromCode = /^Digit([1-9])$/.exec(event.code);
      if (digitFromCode?.[1]) {
        event.preventDefault();
        const digit = Number(digitFromCode[1]);
        void actions.enterDigit(digit, event.shiftKey);
        return;
      }
      if (/^[1-9]$/.test(key)) {
        event.preventDefault();
        void actions.enterDigit(Number(key), event.shiftKey);
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [actions, enabled, game, selectedCell]);
}
