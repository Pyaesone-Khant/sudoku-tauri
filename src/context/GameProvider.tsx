import {
  createContext,
  use,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { tauriSudokuClient, type SudokuClient } from '../api/sudokuClient';
import type {
  ActiveSlot,
  AppStateSnapshot,
  DailyCalendarMonth,
  Difficulty,
  GameSnapshot,
  StatsSummary,
} from '../api/types';
import { playSoundCue } from '../audio/soundEngine';
import { usePreferences } from './PreferencesProvider';

/** Top-level navigation tabs for the desktop shell. */
export type AppTab = 'play' | 'daily' | 'stats';

/** Read-only state slice exposed by `GameProvider`. */
export interface GameStateSlice {
  appState: AppStateSnapshot | null;
  game: GameSnapshot | null;
  selectedCell: number | null;
  notesMode: boolean;
  displaySolveTimeSecs: number;
  activeTab: AppTab;
  errorMessage: string | null;
  /** Digit (`1..=9`) that just reached all 9 placements, for brief completion celebration. */
  celebratingDigit: number | null;
}

/** Mutating actions exposed by `GameProvider`. */
export interface GameActionsSlice {
  setActiveTab: (tab: AppTab) => void;
  selectCell: (index: number | null) => void;
  toggleNotesMode: () => void;
  startStandardGame: (difficulty: Difficulty) => Promise<void>;
  openDailyChallenge: (isoDate: string) => Promise<void>;
  switchActiveSlot: (slot: ActiveSlot) => Promise<void>;
  enterDigit: (digit: number, forceNote?: boolean) => Promise<void>;
  eraseSelected: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  requestHint: () => Promise<void>;
  applyHint: () => Promise<void>;
  dismissHint: () => Promise<void>;
  pauseGame: () => Promise<void>;
  resumeGame: () => Promise<void>;
  loadDailyCalendar: (year: number, month: number) => Promise<DailyCalendarMonth>;
  loadStats: () => Promise<StatsSummary>;
  clearError: () => void;
}

/** Auxiliary metadata exposed by `GameProvider`. */
export interface GameMetaSlice {
  isLoading: boolean;
  isBusy: boolean;
}

/** Compound context value following state / actions / meta separation. */
export interface GameContextValue {
  state: GameStateSlice;
  actions: GameActionsSlice;
  meta: GameMetaSlice;
}

const GameContext = createContext<GameContextValue | null>(null);

interface GameProviderProps {
  client?: SudokuClient;
  children: ReactNode;
}

/**
 * Decouples UI views from Tauri IPC while maintaining authoritative Rust snapshots,
 * active Solve Time ticking, auto-pause listeners, and tactile sound cues.
 */
export function GameProvider({ client = tauriSudokuClient, children }: GameProviderProps) {
  const { muted } = usePreferences();
  const mutedRef = useRef(muted);
  mutedRef.current = muted;

  const [appState, setAppState] = useState<AppStateSnapshot | null>(null);
  const [selectedCell, setSelectedCell] = useState<number | null>(null);
  const [notesMode, setNotesMode] = useState(false);
  const [activeTab, setActiveTabState] = useState<AppTab>('play');
  const [isLoading, setIsLoading] = useState(true);
  const [isBusy, setIsBusy] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [pendingElapsedSecs, setPendingElapsedSecs] = useState(0);
  const [celebratingDigit, setCelebratingDigit] = useState<number | null>(null);

  const pendingElapsedRef = useRef(0);
  pendingElapsedRef.current = pendingElapsedSecs;
  const prevGameRef = useRef<GameSnapshot | null>(null);
  const celebrateTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (celebrateTimerRef.current !== null) {
        window.clearTimeout(celebrateTimerRef.current);
      }
    };
  }, []);

  const applySnapshot = useCallback((snapshot: AppStateSnapshot) => {
    const prevGame = prevGameRef.current;
    const nextGame = snapshot.current_game;
    prevGameRef.current = nextGame;

    setAppState(snapshot);
    setPendingElapsedSecs(0);
    pendingElapsedRef.current = 0;

    // Detect if any digit 1..=9 just crossed from <9 to 9 in the same Game.
    let newlyCompletedDigit: number | null = null;
    if (prevGame && nextGame && prevGame.id === nextGame.id) {
      for (let i = 0; i < 9; i += 1) {
        const before = prevGame.correct_digit_counts[i] ?? 0;
        const after = nextGame.correct_digit_counts[i] ?? 0;
        if (before < 9 && after === 9) {
          newlyCompletedDigit = i + 1;
          break;
        }
      }
    }

    if (newlyCompletedDigit !== null) {
      setCelebratingDigit(newlyCompletedDigit);
      if (celebrateTimerRef.current !== null) {
        window.clearTimeout(celebrateTimerRef.current);
      }
      celebrateTimerRef.current = window.setTimeout(() => {
        setCelebratingDigit(null);
        celebrateTimerRef.current = null;
      }, 900);
    }

    if (snapshot.last_move_outcome === 'Completed') {
      playSoundCue('victory', mutedRef.current);
    } else if (newlyCompletedDigit !== null) {
      playSoundCue('digitComplete', mutedRef.current);
    } else if (snapshot.last_move_outcome === 'Mistake') {
      playSoundCue('mistake', mutedRef.current);
    } else if (snapshot.last_move_outcome === 'Correct') {
      playSoundCue('place', mutedRef.current);
    }
  }, []);

  const flushElapsedTime = useCallback(async () => {
    const delta = pendingElapsedRef.current;
    if (delta <= 0) {
      return;
    }
    pendingElapsedRef.current = 0;
    setPendingElapsedSecs(0);
    try {
      const updated = await client.syncSolveTime(delta);
      setAppState(updated);
    } catch {
      // Ignore transient timer sync failures if game paused concurrently.
    }
  }, [client]);

  // Initial load & auto-pause event subscription.
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    async function init() {
      try {
        const snapshot = await client.getAppState();
        if (active) {
          prevGameRef.current = snapshot.current_game;
          setAppState(snapshot);
          setIsLoading(false);
        }
      } catch (err) {
        if (active) {
          setErrorMessage(err instanceof Error ? err.message : String(err));
          setIsLoading(false);
        }
      }

      try {
        unlisten = await client.onAutoPaused((snapshot) => {
          if (active) {
            setAppState(snapshot);
            setPendingElapsedSecs(0);
            pendingElapsedRef.current = 0;
          }
        });
      } catch {
        // Non-Tauri test environments may not provide an event bus.
      }
    }

    void init();
    return () => {
      active = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, [client]);

  const game = appState?.current_game ?? null;
  const isActivelyPlaying = activeTab === 'play' && game?.status === 'Active';

  // Local 1-second visual timer with periodic 5-second persistence sync to Rust.
  useEffect(() => {
    if (!isActivelyPlaying) {
      return;
    }
    const interval = window.setInterval(() => {
      pendingElapsedRef.current += 1;
      setPendingElapsedSecs(pendingElapsedRef.current);
      if (pendingElapsedRef.current >= 5) {
        void flushElapsedTime();
      }
    }, 1000);
    return () => {
      window.clearInterval(interval);
    };
  }, [isActivelyPlaying, flushElapsedTime]);

  const runMutation = useCallback(
    async (task: () => Promise<AppStateSnapshot>) => {
      setIsBusy(true);
      setErrorMessage(null);
      try {
        await flushElapsedTime();
        const snapshot = await task();
        applySnapshot(snapshot);
      } catch (err) {
        setErrorMessage(err instanceof Error ? err.message : String(err));
      } finally {
        setIsBusy(false);
      }
    },
    [applySnapshot, flushElapsedTime],
  );

  const setActiveTab = useCallback(
    (tab: AppTab) => {
      if (tab !== 'play' && game?.status === 'Active') {
        void runMutation(() => client.pauseGame());
      }
      setActiveTabState(tab);
    },
    [client, game?.status, runMutation],
  );

  const startStandardGame = useCallback(
    async (difficulty: Difficulty) => {
      await runMutation(() => client.startStandardGame(difficulty));
      setSelectedCell(null);
      setNotesMode(false);
      setActiveTabState('play');
    },
    [client, runMutation],
  );

  const openDailyChallenge = useCallback(
    async (isoDate: string) => {
      await runMutation(() => client.openDailyChallenge(isoDate));
      setSelectedCell(null);
      setNotesMode(false);
      setActiveTabState('play');
    },
    [client, runMutation],
  );

  const switchActiveSlot = useCallback(
    async (slot: ActiveSlot) => {
      await runMutation(() => client.switchActiveSlot(slot));
      setSelectedCell(null);
      setActiveTabState('play');
    },
    [client, runMutation],
  );

  const enterDigit = useCallback(
    async (digit: number, forceNote = false) => {
      if (selectedCell === null || !game || game.status !== 'Active') {
        return;
      }
      const target = game.cells[selectedCell];
      if (!target || target.is_given) {
        return;
      }
      const useNote = forceNote || notesMode;
      if (useNote) {
        if (target.value !== null) {
          return;
        }
        playSoundCue('note', mutedRef.current);
        await runMutation(() => client.toggleNote(selectedCell, digit));
      } else {
        await runMutation(() => client.placeDigit(selectedCell, digit));
      }
    },
    [client, game, notesMode, runMutation, selectedCell],
  );

  const eraseSelected = useCallback(async () => {
    if (selectedCell === null || !game || game.status !== 'Active') {
      return;
    }
    const target = game.cells[selectedCell];
    if (!target || target.is_given) {
      return;
    }
    await runMutation(() => client.eraseCell(selectedCell));
  }, [client, game, runMutation, selectedCell]);

  const undo = useCallback(async () => {
    if (!game?.can_undo || game.status !== 'Active') {
      return;
    }
    await runMutation(() => client.undo());
  }, [client, game, runMutation]);

  const redo = useCallback(async () => {
    if (!game?.can_redo || game.status !== 'Active') {
      return;
    }
    await runMutation(() => client.redo());
  }, [client, game, runMutation]);

  const requestHint = useCallback(async () => {
    if (!game || game.status !== 'Active') {
      return;
    }
    setIsBusy(true);
    setErrorMessage(null);
    try {
      await flushElapsedTime();
      const response = await client.requestHint();
      applySnapshot(response.state);
      setSelectedCell(response.hint.cell);
      playSoundCue('hint', mutedRef.current);
    } catch (err) {
      setErrorMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setIsBusy(false);
    }
  }, [applySnapshot, client, flushElapsedTime, game]);

  const applyHint = useCallback(async () => {
    if (!game?.active_hint || game.status !== 'Active') {
      return;
    }
    await runMutation(() => client.applyHint());
  }, [client, game, runMutation]);

  const dismissHint = useCallback(async () => {
    if (!game?.active_hint) {
      return;
    }
    await runMutation(() => client.dismissHint());
  }, [client, game, runMutation]);

  const pauseGame = useCallback(async () => {
    if (!game || game.status !== 'Active') {
      return;
    }
    await runMutation(() => client.pauseGame());
  }, [client, game, runMutation]);

  const resumeGame = useCallback(async () => {
    if (!game || game.status !== 'Paused') {
      return;
    }
    await runMutation(() => client.resumeGame());
  }, [client, game, runMutation]);

  const loadDailyCalendar = useCallback(
    (year: number, month: number) => client.getDailyCalendar(year, month),
    [client],
  );

  const loadStats = useCallback(() => client.getStats(), [client]);

  const toggleNotesMode = useCallback(() => {
    setNotesMode((prev) => !prev);
  }, []);

  const clearError = useCallback(() => {
    setErrorMessage(null);
  }, []);

  const displaySolveTimeSecs = (game?.solve_time_secs ?? 0) + pendingElapsedSecs;

  const value = useMemo<GameContextValue>(
    () => ({
      state: {
        appState,
        game,
        selectedCell,
        notesMode,
        displaySolveTimeSecs,
        activeTab,
        errorMessage,
        celebratingDigit,
      },
      actions: {
        setActiveTab,
        selectCell: setSelectedCell,
        toggleNotesMode,
        startStandardGame,
        openDailyChallenge,
        switchActiveSlot,
        enterDigit,
        eraseSelected,
        undo,
        redo,
        requestHint,
        applyHint,
        dismissHint,
        pauseGame,
        resumeGame,
        loadDailyCalendar,
        loadStats,
        clearError,
      },
      meta: {
        isLoading,
        isBusy,
      },
    }),
    [
      activeTab,
      appState,
      applyHint,
      celebratingDigit,
      clearError,
      dismissHint,
      displaySolveTimeSecs,
      enterDigit,
      eraseSelected,
      errorMessage,
      game,
      isBusy,
      isLoading,
      loadDailyCalendar,
      loadStats,
      notesMode,
      openDailyChallenge,
      pauseGame,
      redo,
      requestHint,
      resumeGame,
      selectedCell,
      setActiveTab,
      startStandardGame,
      switchActiveSlot,
      toggleNotesMode,
      undo,
    ],
  );

  return <GameContext value={value}>{children}</GameContext>;
}

/** Reads the compound `{ state, actions, meta }` game context. */
export function useGame(): GameContextValue {
  const ctx = use(GameContext);
  if (!ctx) {
    throw new Error('useGame must be used within GameProvider');
  }
  return ctx;
}
