import { useEffect, useState } from 'react';
import { Award, Flame, Trophy } from 'lucide-react';
import {
  DIFFICULTIES,
  formatDisplayDate,
  formatSolveTime,
  type Difficulty,
  type StatsSummary,
} from '../../api/types';
import { useGame } from '../../context/GameProvider';

/**
 * Per-Difficulty Statistics, Win Streak, Daily Streak, and Recent History dashboard.
 */
export function StatsView() {
  const { actions } = useGame();
  const [selectedDifficulty, setSelectedDifficulty] = useState<Difficulty>('Easy');
  const [stats, setStats] = useState<StatsSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;
    setLoading(true);
    void actions
      .loadStats()
      .then((data) => {
        if (active) {
          setStats(data);
          setLoading(false);
        }
      })
      .catch(() => {
        if (active) {
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [actions]);

  if (loading || !stats) {
    return (
      <div className="flex flex-1 items-center justify-center text-[13px] text-[var(--ink-secondary)]">
        Loading statistics…
      </div>
    );
  }

  const activeDiffStats = stats.by_difficulty.find((d) => d.difficulty === selectedDifficulty) ?? {
    difficulty: selectedDifficulty,
    games_completed: 0,
    games_abandoned: 0,
    best_score: null,
    avg_score: null,
    best_time_secs: null,
    avg_time_secs: null,
    flawless_solves: 0,
  };

  return (
    <section className="mx-auto flex w-full max-w-[740px] flex-1 flex-col gap-4 px-5 py-5">
      {/* Top Streak Strip: Win Streak vs Daily Streak */}
      <div className="grid grid-cols-2 gap-3">
        <div className="flex items-center justify-between rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] px-4 py-3 shadow-xs">
          <div className="flex items-center gap-2.5">
            <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-[var(--accent-soft)] text-[var(--accent)]">
              <Trophy className="h-4 w-4" />
            </span>
            <div>
              <div className="text-[12px] font-medium text-[var(--ink-secondary)]">Win Streak</div>
              <div className="text-[11px] text-[var(--ink-muted)]">Standard Games</div>
            </div>
          </div>
          <div className="text-right">
            <div className="tabular-nums text-[20px] font-bold leading-tight text-[var(--ink-primary)]">
              {stats.win_streak_current}
            </div>
            <div className="tabular-nums text-[11px] text-[var(--ink-secondary)]">
              Best {stats.win_streak_best}
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] px-4 py-3 shadow-xs">
          <div className="flex items-center gap-2.5">
            <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-[var(--cell-hint)] text-[var(--ink-gold)]">
              <Flame className="h-4 w-4" />
            </span>
            <div>
              <div className="text-[12px] font-medium text-[var(--ink-secondary)]">Daily Streak</div>
              <div className="text-[11px] text-[var(--ink-muted)]">Daily Challenges</div>
            </div>
          </div>
          <div className="text-right">
            <div className="tabular-nums text-[20px] font-bold leading-tight text-[var(--ink-primary)]">
              {stats.daily_streak_current}
            </div>
            <div className="tabular-nums text-[11px] text-[var(--ink-secondary)]">
              Best {stats.daily_streak_best}
            </div>
          </div>
        </div>
      </div>

      {/* Per-Difficulty Segmented Panel */}
      <div className="rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-4 shadow-xs">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h1 className="text-[14px] font-semibold text-[var(--ink-primary)]">
            Performance by Difficulty
          </h1>
          <div
            role="tablist"
            aria-label="Difficulty statistics"
            className="flex items-center rounded-lg bg-[var(--surface-subtle)] p-0.5"
          >
            {DIFFICULTIES.map((diff) => {
              const selected = diff === selectedDifficulty;
              return (
                <button
                  key={diff}
                  type="button"
                  role="tab"
                  aria-selected={selected}
                  onClick={() => setSelectedDifficulty(diff)}
                  className={`rounded-md px-3 py-1 text-[12px] font-medium transition-colors ${
                    selected
                      ? 'bg-[var(--surface-elevated)] text-[var(--ink-primary)] shadow-xs'
                      : 'text-[var(--ink-secondary)] hover:text-[var(--ink-primary)]'
                  }`}
                >
                  {diff}
                </button>
              );
            })}
          </div>
        </div>

        <div className="mt-4 grid grid-cols-3 gap-3 sm:grid-cols-6">
          <MetricCell label="Completed" value={String(activeDiffStats.games_completed)} />
          <MetricCell label="Flawless Solves" value={String(activeDiffStats.flawless_solves)} />
          <MetricCell
            label="Best Score"
            value={
              activeDiffStats.best_score !== null
                ? activeDiffStats.best_score.toLocaleString()
                : '—'
            }
          />
          <MetricCell
            label="Avg Score"
            value={
              activeDiffStats.avg_score !== null ? activeDiffStats.avg_score.toLocaleString() : '—'
            }
          />
          <MetricCell
            label="Best Time"
            value={
              activeDiffStats.best_time_secs !== null
                ? formatSolveTime(activeDiffStats.best_time_secs)
                : '—'
            }
          />
          <MetricCell
            label="Avg Time"
            value={
              activeDiffStats.avg_time_secs !== null
                ? formatSolveTime(activeDiffStats.avg_time_secs)
                : '—'
            }
          />
        </div>
      </div>

      {/* Recent Completed History */}
      <div className="flex-1 rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-4 shadow-xs">
        <h2 className="text-[14px] font-semibold text-[var(--ink-primary)]">Recent History</h2>

        {stats.recent_history.length === 0 ? (
          <p className="mt-4 text-center text-[12px] text-[var(--ink-secondary)]">
            Complete your first Puzzle to see your Final Scores and Solve Times here.
          </p>
        ) : (
          <div className="mt-3 overflow-x-auto">
            <table className="w-full text-left text-[12px]">
              <thead>
                <tr className="border-b border-[var(--border-hairline)] text-[11px] font-medium text-[var(--ink-secondary)]">
                  <th className="pb-2 font-medium">Game</th>
                  <th className="pb-2 font-medium">Difficulty</th>
                  <th className="pb-2 text-right font-medium">Solve Time</th>
                  <th className="pb-2 text-right font-medium">Mistakes</th>
                  <th className="pb-2 text-right font-medium">Hints</th>
                  <th className="pb-2 text-right font-medium">Final Score</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--border-hairline)]">
                {stats.recent_history.map((record) => (
                  <tr key={record.id} className="text-[var(--ink-primary)]">
                    <td className="py-2 pr-3">
                      <div className="flex items-center gap-1.5">
                        <span className="font-medium">
                          {record.kind.type === 'Daily'
                            ? `Daily · ${formatDisplayDate(record.kind.date)}`
                            : 'Standard'}
                        </span>
                        {record.flawless ? (
                          <Award
                            aria-label="Flawless Solve"
                            className="h-3.5 w-3.5 text-[var(--ink-gold)]"
                          />
                        ) : null}
                      </div>
                    </td>
                    <td className="py-2 pr-3 text-[var(--ink-secondary)]">{record.difficulty}</td>
                    <td className="tabular-nums py-2 pr-3 text-right">
                      {formatSolveTime(record.solve_time_secs)}
                    </td>
                    <td className="tabular-nums py-2 pr-3 text-right">{record.mistakes}</td>
                    <td className="tabular-nums py-2 pr-3 text-right">{record.hints_used}</td>
                    <td className="tabular-nums py-2 text-right font-semibold">
                      {record.score.final_score.toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}

function MetricCell({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl bg-[var(--surface-canvas)] px-3 py-2.5">
      <div className="text-[11px] text-[var(--ink-secondary)]">{label}</div>
      <div className="tabular-nums mt-1 text-[16px] font-semibold text-[var(--ink-primary)]">
        {value}
      </div>
    </div>
  );
}
