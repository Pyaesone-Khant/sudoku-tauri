import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ActiveSlot,
  AppStateSnapshot,
  DailyCalendarMonth,
  Difficulty,
  HintResponse,
  StatsSummary,
} from './types';

/**
 * Abstract IPC contract allowing UI components and tests to swap the Tauri backend cleanly.
 */
export interface SudokuClient {
  getAppState(): Promise<AppStateSnapshot>;
  startStandardGame(difficulty: Difficulty): Promise<AppStateSnapshot>;
  openDailyChallenge(isoDate: string): Promise<AppStateSnapshot>;
  switchActiveSlot(slot: ActiveSlot): Promise<AppStateSnapshot>;
  placeDigit(cell: number, digit: number): Promise<AppStateSnapshot>;
  toggleNote(cell: number, digit: number): Promise<AppStateSnapshot>;
  eraseCell(cell: number): Promise<AppStateSnapshot>;
  undo(): Promise<AppStateSnapshot>;
  redo(): Promise<AppStateSnapshot>;
  requestHint(): Promise<HintResponse>;
  applyHint(): Promise<AppStateSnapshot>;
  dismissHint(): Promise<AppStateSnapshot>;
  pauseGame(): Promise<AppStateSnapshot>;
  resumeGame(): Promise<AppStateSnapshot>;
  syncSolveTime(deltaSecs: number): Promise<AppStateSnapshot>;
  getDailyCalendar(year: number, month: number): Promise<DailyCalendarMonth>;
  getStats(): Promise<StatsSummary>;
  onAutoPaused(callback: (snapshot: AppStateSnapshot) => void): Promise<UnlistenFn>;
}

/**
 * Production Tauri v2 IPC client delegating all domain commands to Rust.
 */
export const tauriSudokuClient: SudokuClient = {
  getAppState: () => invoke<AppStateSnapshot>('get_app_state'),
  startStandardGame: (difficulty) =>
    invoke<AppStateSnapshot>('start_standard_game', { difficulty }),
  openDailyChallenge: (isoDate) =>
    invoke<AppStateSnapshot>('open_daily_challenge', { date: isoDate }),
  switchActiveSlot: (slot) =>
    invoke<AppStateSnapshot>('switch_active_slot', { slot }),
  placeDigit: (cell, digit) =>
    invoke<AppStateSnapshot>('place_digit', { cell, digit }),
  toggleNote: (cell, digit) =>
    invoke<AppStateSnapshot>('toggle_note', { cell, digit }),
  eraseCell: (cell) => invoke<AppStateSnapshot>('erase_cell', { cell }),
  undo: () => invoke<AppStateSnapshot>('undo'),
  redo: () => invoke<AppStateSnapshot>('redo'),
  requestHint: () => invoke<HintResponse>('request_hint'),
  applyHint: () => invoke<AppStateSnapshot>('apply_hint'),
  dismissHint: () => invoke<AppStateSnapshot>('dismiss_hint'),
  pauseGame: () => invoke<AppStateSnapshot>('pause_game'),
  resumeGame: () => invoke<AppStateSnapshot>('resume_game'),
  syncSolveTime: (deltaSecs) =>
    invoke<AppStateSnapshot>('sync_solve_time', { deltaSecs }),
  getDailyCalendar: (year, month) =>
    invoke<DailyCalendarMonth>('get_daily_calendar', { year, month }),
  getStats: () => invoke<StatsSummary>('get_stats'),
  onAutoPaused: (callback) =>
    listen<AppStateSnapshot>('game-auto-paused', (event) => {
      callback(event.payload);
    }),
};
