import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { SudokuClient } from './api/sudokuClient';
import type { AppStateSnapshot, CellSnapshot, GameSnapshot } from './api/types';
import { App } from './App';

function buildCells(overrides: Partial<CellSnapshot>[] = []): CellSnapshot[] {
  return Array.from({ length: 81 }, (_, index) => ({
    index,
    value: index === 0 ? 5 : null,
    is_given: index === 0,
    is_mistake: false,
    is_hinted: false,
    notes: [],
    ...overrides[index],
  }));
}

function buildGame(partial: Partial<GameSnapshot> = {}): GameSnapshot {
  return {
    id: 1,
    kind: { type: 'Standard' },
    difficulty: 'Easy',
    status: 'Active',
    cells: buildCells(),
    mistakes: 0,
    hints_used: 0,
    solve_time_secs: 45,
    score: {
      earned_score: 0,
      time_bonus: 0,
      mistake_penalty: 0,
      hint_penalty: 0,
      total_penalty: 0,
      final_score: 0,
    },
    flawless: true,
    active_hint: null,
    can_undo: false,
    can_redo: false,
    correct_digit_counts: [0, 0, 0, 0, 1, 0, 0, 0, 0],
    ...partial,
  };
}

function createMockClient(initial: AppStateSnapshot): SudokuClient {
  let current = initial;
  return {
    getAppState: vi.fn(async () => current),
    startStandardGame: vi.fn(async (difficulty) => {
      current = {
        ...current,
        active_slot: 'Standard',
        has_in_progress_standard: true,
        current_game: buildGame({ difficulty }),
        last_move_outcome: null,
      };
      return current;
    }),
    openDailyChallenge: vi.fn(async () => current),
    switchActiveSlot: vi.fn(async () => current),
    placeDigit: vi.fn(async (cell, digit) => {
      const cells = (current.current_game?.cells ?? buildCells()).map((c) =>
        c.index === cell ? { ...c, value: digit, is_mistake: digit === 9 } : c,
      );
      const isMistake = digit === 9;
      const counts = [...(current.current_game?.correct_digit_counts ?? [0, 0, 0, 0, 1, 0, 0, 0, 0])];
      if (!isMistake && digit >= 1 && digit <= 9) {
        counts[digit - 1] = Math.min(9, (counts[digit - 1] ?? 0) + 1);
      }
      current = {
        ...current,
        current_game: buildGame({
          ...current.current_game,
          cells,
          correct_digit_counts: counts,
          mistakes: (current.current_game?.mistakes ?? 0) + (isMistake ? 1 : 0),
          can_undo: true,
          score: {
            earned_score: isMistake ? 0 : 50,
            time_bonus: 0,
            mistake_penalty: isMistake ? 100 : 0,
            hint_penalty: 0,
            total_penalty: isMistake ? 100 : 0,
            final_score: isMistake ? 0 : 50,
          },
        }),
        last_move_outcome: isMistake ? 'Mistake' : 'Correct',
      };
      return current;
    }),
    toggleNote: vi.fn(async (cell, digit) => {
      const cells = (current.current_game?.cells ?? buildCells()).map((c) =>
        c.index === cell ? { ...c, notes: [digit] } : c,
      );
      current = {
        ...current,
        current_game: buildGame({ ...current.current_game, cells, can_undo: true }),
      };
      return current;
    }),
    eraseCell: vi.fn(async () => current),
    undo: vi.fn(async () => current),
    redo: vi.fn(async () => current),
    requestHint: vi.fn(async () => {
      const hint = {
        kind: 'PlaceDigit' as const,
        cell: 1,
        digit: 3,
        technique: 'HiddenSingle' as const,
        explanation: 'Hidden Single: 3 can only go in Row 1, Column 2.',
        highlight_cells: [1],
      };
      current = {
        ...current,
        current_game: buildGame({
          ...current.current_game,
          hints_used: 1,
          active_hint: hint,
        }),
      };
      return { state: current, hint };
    }),
    applyHint: vi.fn(async () => {
      const cells = (current.current_game?.cells ?? buildCells()).map((c) =>
        c.index === 1 ? { ...c, value: 3, is_hinted: true } : c,
      );
      current = {
        ...current,
        current_game: buildGame({
          ...current.current_game,
          cells,
          active_hint: null,
        }),
        last_move_outcome: 'Correct',
      };
      return current;
    }),
    dismissHint: vi.fn(async () => current),
    pauseGame: vi.fn(async () => {
      current = {
        ...current,
        current_game: buildGame({ ...current.current_game, status: 'Paused' }),
      };
      return current;
    }),
    resumeGame: vi.fn(async () => {
      current = {
        ...current,
        current_game: buildGame({ ...current.current_game, status: 'Active' }),
      };
      return current;
    }),
    syncSolveTime: vi.fn(async () => current),
    getDailyCalendar: vi.fn(async () => ({
      year: 2026,
      month: 11,
      today: { year: 2026, month: 11, day: 19 },
      daily_streak_current: 3,
      daily_streak_best: 5,
      days: [
        {
          date: { year: 2026, month: 11, day: 18 },
          difficulty: 'Medium' as const,
          state: { status: 'Playable' as const },
        },
        {
          date: { year: 2026, month: 11, day: 19 },
          difficulty: 'Hard' as const,
          state: { status: 'Playable' as const },
        },
        {
          date: { year: 2026, month: 11, day: 20 },
          difficulty: 'Hard' as const,
          state: { status: 'FutureLocked' as const },
        },
      ],
    })),
    getStats: vi.fn(async () => ({
      by_difficulty: [
        {
          difficulty: 'Easy' as const,
          games_completed: 4,
          games_abandoned: 1,
          best_score: 2600,
          avg_score: 2300,
          best_time_secs: 120,
          avg_time_secs: 180,
          flawless_solves: 2,
        },
        {
          difficulty: 'Hard' as const,
          games_completed: 1,
          games_abandoned: 0,
          best_score: 4100,
          avg_score: 4100,
          best_time_secs: 420,
          avg_time_secs: 420,
          flawless_solves: 1,
        },
      ],
      win_streak_current: 2,
      win_streak_best: 4,
      daily_streak_current: 3,
      daily_streak_best: 5,
      recent_history: [
        {
          id: 9,
          kind: { type: 'Daily' as const, date: { year: 2026, month: 11, day: 19 } },
          difficulty: 'Hard' as const,
          solve_time_secs: 420,
          mistakes: 0,
          hints_used: 0,
          score: {
            earned_score: 3000,
            time_bonus: 1100,
            mistake_penalty: 0,
            hint_penalty: 0,
            total_penalty: 0,
            final_score: 4100,
          },
          flawless: true,
          completed_on: { year: 2026, month: 11, day: 19 },
        },
      ],
    })),
    onAutoPaused: vi.fn(async () => () => {}),
  };
}

describe('Sudoku Play View', () => {
  it('starts a Standard Game from welcome screen and places a digit via keyboard', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: null,
      has_in_progress_standard: false,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await user.click(await screen.findByRole('button', { name: /Easy/i }));

    await waitFor(() => {
      expect(screen.getByRole('grid', { name: 'Sudoku board' })).toBeInTheDocument();
    });

    // Select editable cell (Row 1, Column 2) and press '3'.
    await user.click(screen.getByRole('button', { name: /Row 1, Column 2, empty/i }));
    await user.keyboard('3');

    await waitFor(() => {
      expect(client.placeDigit).toHaveBeenCalledWith(1, 3);
    });
  });

  it('hides board values behind Pause veil and reveals them on Resume', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: buildGame(),
      has_in_progress_standard: true,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await screen.findByRole('grid', { name: 'Sudoku board' });

    await user.click(screen.getByRole('button', { name: 'Pause' }));
    const pausedDialog = await screen.findByRole('dialog', { name: 'Game paused' });
    expect(pausedDialog).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Row 1, Column 1, hidden while paused/i })).toBeInTheDocument();

    await user.click(within(pausedDialog).getByRole('button', { name: 'Resume' }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Game paused' })).not.toBeInTheDocument();
    });
  });

  it('displays full score breakdown and Flawless Solve badge upon completion', async () => {
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: buildGame({
        status: 'Completed',
        solve_time_secs: 150,
        flawless: true,
        score: {
          earned_score: 2000,
          time_bonus: 500,
          mistake_penalty: 0,
          hint_penalty: 0,
          total_penalty: 0,
          final_score: 2500,
        },
      }),
      has_in_progress_standard: false,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: 'Completed',
    });

    render(<App client={client} />);
    const dialog = await screen.findByRole('dialog', { name: 'Puzzle completed' });
    expect(screen.getByTestId('victory-confetti')).toBeInTheDocument();
    expect(within(dialog).getByText('Flawless Solve')).toBeInTheDocument();
    expect(within(dialog).getByText('2,500')).toBeInTheDocument();
    expect(within(dialog).getByText('+2,000')).toBeInTheDocument();
    expect(within(dialog).getByText('+500')).toBeInTheDocument();
  });

  it('requests and applies a Smart Hint showing the technique explanation', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: buildGame(),
      has_in_progress_standard: true,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await screen.findByRole('grid', { name: 'Sudoku board' });

    await user.click(screen.getByRole('button', { name: 'Hint' }));
    expect(await screen.findByText('Hidden Single')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Apply 3' }));
    await waitFor(() => {
      expect(client.applyHint).toHaveBeenCalledTimes(1);
    });
  });

  it('opens a past catch-up date from the Daily Challenge Calendar and locks future dates', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: null,
      has_in_progress_standard: false,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await user.click(screen.getByRole('button', { name: 'Daily' }));

    // Select past date Nov 18 and click Play Daily Challenge
    await user.click(await screen.findByRole('button', { name: /Nov 18, 2026/i }));
    await user.click(screen.getByRole('button', { name: 'Play Daily Challenge' }));
    expect(client.openDailyChallenge).toHaveBeenCalledWith('2026-11-18');
  });

  it('switches Difficulty tabs on the Stats view and displays Win Streak, Daily Streak, and Recent History', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: null,
      has_in_progress_standard: false,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await user.click(screen.getByRole('button', { name: 'Stats' }));

    expect(await screen.findByText('Win Streak')).toBeInTheDocument();
    expect(screen.getByText('Daily Streak')).toBeInTheDocument();
    expect(screen.getByText('2,600')).toBeInTheDocument();

    await user.click(screen.getByRole('tab', { name: 'Hard' }));
    expect(screen.getAllByText('4,100').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText(/Daily · Nov 19, 2026/)).toBeInTheDocument();
  });

  it('triggers micro-confetti on the NumberPad when a digit completes all 9 placements', async () => {
    const user = userEvent.setup();
    const client = createMockClient({
      active_slot: 'Standard',
      current_game: buildGame({
        // Digit 3 has 8 placed; digit 5 already has 9 placed (must not misfire on load).
        correct_digit_counts: [0, 0, 8, 0, 9, 0, 0, 0, 0],
      }),
      has_in_progress_standard: true,
      in_progress_daily_date: null,
      today: { year: 2026, month: 11, day: 19 },
      last_move_outcome: null,
    });

    render(<App client={client} />);
    await screen.findByRole('grid', { name: 'Sudoku board' });

    expect(screen.queryByTestId('digit-confetti-5')).not.toBeInTheDocument();
    expect(screen.queryByTestId('digit-confetti-3')).not.toBeInTheDocument();

    // Place the 9th copy of digit 3 -> triggers micro-confetti for digit 3.
    await user.click(screen.getByRole('button', { name: /Row 1, Column 2, empty/i }));
    await user.keyboard('3');

    expect(await screen.findByTestId('digit-confetti-3')).toBeInTheDocument();
  });
});
