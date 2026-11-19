/**
 * Authoritative DTO types mirroring `sudoku-core` snapshots over Tauri IPC.
 * All names follow the canonical glossary in `CONTEXT.md`.
 */

/** Skill rating of a Puzzle graded by human solving technique. */
export type Difficulty = 'Easy' | 'Medium' | 'Hard' | 'Expert';

/** Canonical list of all four Difficulties in ascending order. */
export const DIFFICULTIES: readonly Difficulty[] = ['Easy', 'Medium', 'Hard', 'Expert'];

/** Local calendar date components matching Rust `DailyDate`. */
export interface DailyDate {
  year: number;
  month: number;
  day: number;
}

/** Distinguishes a Standard Game from a date-seeded Daily Challenge. */
export type GameKind =
  | { type: 'Standard' }
  | { type: 'Daily'; date: DailyDate };

/** Lifecycle state of a Game. */
export type GameStatus = 'Active' | 'Paused' | 'Completed' | 'Abandoned';

/** Identifies which active Game slot is focused on the Play screen. */
export type ActiveSlot = 'Standard' | 'Daily';

/** Outcome returned after placing a digit or applying a Hint. */
export type MoveOutcome = 'NoChange' | 'Correct' | 'Mistake' | 'Completed';

/** Human solving technique identifier used by Smart Hints. */
export type SolvingTechnique =
  | 'NakedSingle'
  | 'HiddenSingle'
  | 'LockedCandidates'
  | 'NakedPair'
  | 'HiddenPair'
  | 'NakedTriple'
  | 'HiddenTriple'
  | 'XWing'
  | 'XYWing'
  | 'Swordfish'
  | 'ForcingChain';

/** Smart Hint suggestion produced by the Rust human-technique solver. */
export type HintSuggestion =
  | {
      kind: 'FixMistake';
      cell: number;
      wrong_digit: number;
      explanation: string;
    }
  | {
      kind: 'PlaceDigit';
      cell: number;
      digit: number;
      technique: SolvingTechnique;
      explanation: string;
      highlight_cells: number[];
    };

/** Complete score breakdown for a live or completed Game. */
export interface ScoreBreakdown {
  earned_score: number;
  time_bonus: number;
  mistake_penalty: number;
  hint_penalty: number;
  total_penalty: number;
  final_score: number;
}

/** Snapshot of a single cell on the 9x9 board. */
export interface CellSnapshot {
  index: number;
  value: number | null;
  is_given: boolean;
  is_mistake: boolean;
  is_hinted: boolean;
  notes: number[];
}

/** Authoritative snapshot of a Game. */
export interface GameSnapshot {
  id: number;
  kind: GameKind;
  difficulty: Difficulty;
  status: GameStatus;
  cells: CellSnapshot[];
  mistakes: number;
  hints_used: number;
  solve_time_secs: number;
  score: ScoreBreakdown;
  flawless: boolean;
  active_hint: HintSuggestion | null;
  can_undo: boolean;
  can_redo: boolean;
  correct_digit_counts: number[];
}

/** Top-level application state returned by mutating commands. */
export interface AppStateSnapshot {
  active_slot: ActiveSlot;
  current_game: GameSnapshot | null;
  has_in_progress_standard: boolean;
  in_progress_daily_date: DailyDate | null;
  today: DailyDate;
  last_move_outcome: MoveOutcome | null;
}

/** Response returned when requesting a Smart Hint. */
export interface HintResponse {
  state: AppStateSnapshot;
  hint: HintSuggestion;
}

/** Immutable completion record stored in local history. */
export interface CompletedGameRecord {
  id: number;
  kind: GameKind;
  difficulty: Difficulty;
  solve_time_secs: number;
  mistakes: number;
  hints_used: number;
  score: ScoreBreakdown;
  flawless: boolean;
  completed_on: DailyDate;
}

/** Aggregated statistics for a single Difficulty tier. */
export interface DifficultyStats {
  difficulty: Difficulty;
  games_completed: number;
  games_abandoned: number;
  best_score: number | null;
  avg_score: number | null;
  best_time_secs: number | null;
  avg_time_secs: number | null;
  flawless_solves: number;
}

/** Full Statistics dashboard projection. */
export interface StatsSummary {
  by_difficulty: DifficultyStats[];
  win_streak_current: number;
  win_streak_best: number;
  daily_streak_current: number;
  daily_streak_best: number;
  recent_history: CompletedGameRecord[];
}

/** Status of a single day cell inside the Daily Challenge Calendar. */
export type DailyDayState =
  | { status: 'FutureLocked' }
  | { status: 'Playable' }
  | { status: 'InProgress' }
  | { status: 'Completed'; record: CompletedGameRecord };

/** Single day cell in the Daily Challenge Calendar. */
export interface DailyCalendarDay {
  date: DailyDate;
  difficulty: Difficulty;
  state: DailyDayState;
}

/** Monthly Daily Challenge Calendar projection. */
export interface DailyCalendarMonth {
  year: number;
  month: number;
  today: DailyDate;
  daily_streak_current: number;
  daily_streak_best: number;
  days: DailyCalendarDay[];
}

/** Formats a `DailyDate` as canonical `YYYY-MM-DD`. */
export function formatIsoDate(date: DailyDate): string {
  const y = String(date.year).padStart(4, '0');
  const m = String(date.month).padStart(2, '0');
  const d = String(date.day).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** Formats a `DailyDate` in readable form (e.g., `Nov 19, 2026`). */
export function formatDisplayDate(date: DailyDate): string {
  const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const monthLabel = months[date.month - 1] ?? String(date.month);
  return `${monthLabel} ${date.day}, ${date.year}`;
}

/** Formats elapsed seconds as `MM:SS` (or `H:MM:SS` over an hour). */
export function formatSolveTime(totalSecs: number): string {
  const safe = Math.max(0, Math.floor(totalSecs));
  const hours = Math.floor(safe / 3600);
  const mins = Math.floor((safe % 3600) / 60);
  const secs = safe % 60;
  const mm = String(mins).padStart(2, '0');
  const ss = String(secs).padStart(2, '0');
  return hours > 0 ? `${hours}:${mm}:${ss}` : `${mm}:${ss}`;
}

/** Maps a `SolvingTechnique` identifier to its human-readable label. */
export function formatTechniqueName(technique: SolvingTechnique): string {
  const labels: Record<SolvingTechnique, string> = {
    NakedSingle: 'Naked Single',
    HiddenSingle: 'Hidden Single',
    LockedCandidates: 'Locked Candidates',
    NakedPair: 'Naked Pair',
    HiddenPair: 'Hidden Pair',
    NakedTriple: 'Naked Triple',
    HiddenTriple: 'Hidden Triple',
    XWing: 'X-Wing',
    XYWing: 'XY-Wing',
    Swordfish: 'Swordfish',
    ForcingChain: 'Forcing Chain',
  };
  return labels[technique];
}
