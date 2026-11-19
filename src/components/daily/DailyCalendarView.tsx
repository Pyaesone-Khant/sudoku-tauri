import { useEffect, useMemo, useState } from 'react';
import { Award, Check, ChevronLeft, ChevronRight, Flame, Lock, Play } from 'lucide-react';
import {
  formatDisplayDate,
  formatIsoDate,
  formatSolveTime,
  type DailyCalendarDay,
  type DailyCalendarMonth,
} from '../../api/types';
import { useGame } from '../../context/GameProvider';

const WEEKDAY_HEADERS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as const;
const MONTH_NAMES = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
] as const;

/** Computes Monday-first weekday offset (0 = Mon .. 6 = Sun) for the 1st of a month. */
function firstWeekdayOffset(year: number, month: number): number {
  const jsDay = new Date(year, month - 1, 1).getDay(); // 0 = Sun .. 6 = Sat
  return (jsDay + 6) % 7;
}

/**
 * Interactive offline Daily Challenge monthly calendar with catch-up play and Daily Streak.
 */
export function DailyCalendarView() {
  const { state, actions, meta } = useGame();
  const today = state.appState?.today ?? { year: 2026, month: 11, day: 19 };

  const [cursor, setCursor] = useState({ year: today.year, month: today.month });
  const [calendar, setCalendar] = useState<DailyCalendarMonth | null>(null);
  const [selectedDayNum, setSelectedDayNum] = useState<number>(today.day);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;
    setLoading(true);
    void actions
      .loadDailyCalendar(cursor.year, cursor.month)
      .then((monthData) => {
        if (active) {
          setCalendar(monthData);
          const isCurrentMonth =
            cursor.year === monthData.today.year && cursor.month === monthData.today.month;
          setSelectedDayNum((prev) => {
            if (isCurrentMonth && prev > monthData.days.length) {
              return monthData.today.day;
            }
            return Math.min(Math.max(1, prev), monthData.days.length);
          });
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
  }, [actions, cursor.month, cursor.year]);

  const canGoNextMonth =
    cursor.year < today.year || (cursor.year === today.year && cursor.month < today.month);

  const goPrevMonth = () => {
    setSelectedDayNum(1);
    setCursor((prev) =>
      prev.month === 1
        ? { year: prev.year - 1, month: 12 }
        : { year: prev.year, month: prev.month - 1 },
    );
  };

  const goNextMonth = () => {
    if (!canGoNextMonth) {
      return;
    }
    setSelectedDayNum(1);
    setCursor((prev) =>
      prev.month === 12
        ? { year: prev.year + 1, month: 1 }
        : { year: prev.year, month: prev.month + 1 },
    );
  };

  const selectedDay: DailyCalendarDay | null = useMemo(() => {
    if (!calendar) {
      return null;
    }
    return calendar.days.find((d) => d.date.day === selectedDayNum) ?? calendar.days[0] ?? null;
  }, [calendar, selectedDayNum]);

  const leadingBlanks = firstWeekdayOffset(cursor.year, cursor.month);
  const monthLabel = `${MONTH_NAMES[cursor.month - 1] ?? ''} ${cursor.year}`;

  return (
    <section className="mx-auto flex w-full max-w-[780px] flex-1 flex-col justify-center gap-4 px-5 py-5">
      <div className="flex flex-wrap items-start justify-center gap-6">
        {/* Monthly Calendar Grid */}
        <div className="w-full max-w-[420px] rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-4 shadow-xs">
          <div className="mb-3 flex items-center justify-between">
            <h1 className="text-[15px] font-semibold tracking-tight text-[var(--ink-primary)]">
              {monthLabel}
            </h1>
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={goPrevMonth}
                aria-label="Previous month"
                className="flex h-7 w-7 items-center justify-center rounded-md text-[var(--ink-secondary)] hover:bg-[var(--surface-subtle)] hover:text-[var(--ink-primary)]"
              >
                <ChevronLeft className="h-4 w-4" />
              </button>
              <button
                type="button"
                disabled={!canGoNextMonth}
                onClick={goNextMonth}
                aria-label="Next month"
                className="flex h-7 w-7 items-center justify-center rounded-md text-[var(--ink-secondary)] hover:bg-[var(--surface-subtle)] hover:text-[var(--ink-primary)] disabled:opacity-30"
              >
                <ChevronRight className="h-4 w-4" />
              </button>
            </div>
          </div>

          <div className="mb-1.5 grid grid-cols-7 text-center text-[11px] font-medium text-[var(--ink-secondary)]">
            {WEEKDAY_HEADERS.map((day) => (
              <div key={day} className="py-1">
                {day}
              </div>
            ))}
          </div>

          {loading || !calendar ? (
            <div className="flex h-56 items-center justify-center text-[12px] text-[var(--ink-secondary)]">
              Loading calendar…
            </div>
          ) : (
            <div className="grid grid-cols-7 gap-1.5" role="grid" aria-label="Daily Challenge Calendar">
              {Array.from({ length: leadingBlanks }).map((_, i) => (
                <div key={`blank-${i}`} />
              ))}

              {calendar.days.map((entry) => {
                const isSelected = selectedDay?.date.day === entry.date.day;
                const isToday =
                  entry.date.year === calendar.today.year &&
                  entry.date.month === calendar.today.month &&
                  entry.date.day === calendar.today.day;
                const isCompleted = entry.state.status === 'Completed';
                const isFuture = entry.state.status === 'FutureLocked';
                const isInProgress = entry.state.status === 'InProgress';
                const isFlawless =
                  entry.state.status === 'Completed' && entry.state.record.flawless;

                let cellStyle =
                  'bg-[var(--surface-canvas)] text-[var(--ink-primary)] hover:bg-[var(--surface-subtle)]';
                if (isCompleted) {
                  cellStyle = 'bg-[var(--cell-hint)] text-[var(--ink-success)] font-semibold';
                } else if (isFuture) {
                  cellStyle = 'bg-transparent text-[var(--ink-muted)] opacity-55';
                } else if (isInProgress) {
                  cellStyle = 'bg-[var(--accent-soft)] text-[var(--accent)] font-semibold';
                }

                return (
                  <button
                    key={entry.date.day}
                    type="button"
                    onClick={() => setSelectedDayNum(entry.date.day)}
                    aria-label={`${formatDisplayDate(entry.date)}, ${entry.difficulty}, ${entry.state.status}`}
                    aria-pressed={isSelected}
                    className={`relative flex aspect-square flex-col items-center justify-center rounded-lg text-[12px] transition-colors ${cellStyle} ${
                      isSelected
                        ? 'ring-2 ring-[var(--accent)]'
                        : isToday
                          ? 'ring-1 ring-[var(--border-strong)]'
                          : ''
                    }`}
                  >
                    <span className="tabular-nums leading-none">{entry.date.day}</span>
                    <span className="mt-1 flex h-2.5 items-center justify-center">
                      {isCompleted ? (
                        isFlawless ? (
                          <Award className="h-2.5 w-2.5 text-[var(--ink-gold)]" />
                        ) : (
                          <Check className="h-2.5 w-2.5" />
                        )
                      ) : isFuture ? (
                        <Lock className="h-2 w-2" />
                      ) : isInProgress ? (
                        <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent)]" />
                      ) : null}
                    </span>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Right Column: Daily Streak + Selected Day Summary */}
        <aside className="flex w-full max-w-[260px] flex-col gap-3.5">
          <div className="rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-4 shadow-xs">
            <div className="flex items-center gap-2 text-[12px] font-medium text-[var(--ink-secondary)]">
              <Flame className="h-4 w-4 text-[var(--ink-gold)]" />
              Daily Streak
            </div>
            <div className="mt-2 grid grid-cols-2 gap-3">
              <div>
                <div className="tabular-nums text-[24px] font-bold leading-tight text-[var(--ink-primary)]">
                  {calendar?.daily_streak_current ?? 0}
                </div>
                <div className="text-[11px] text-[var(--ink-secondary)]">Current</div>
              </div>
              <div>
                <div className="tabular-nums text-[24px] font-bold leading-tight text-[var(--ink-primary)]">
                  {calendar?.daily_streak_best ?? 0}
                </div>
                <div className="text-[11px] text-[var(--ink-secondary)]">Best</div>
              </div>
            </div>
          </div>

          {selectedDay ? (
            <div className="rounded-2xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-4 shadow-xs">
              <div className="text-[15px] font-semibold text-[var(--ink-primary)]">
                {formatDisplayDate(selectedDay.date)}
              </div>
              <div className="mt-0.5 text-[12px] text-[var(--ink-secondary)]">
                Difficulty: <span className="font-medium text-[var(--ink-primary)]">{selectedDay.difficulty}</span>
              </div>

              {selectedDay.state.status === 'Completed' ? (
                <div className="mt-4 space-y-2 border-t border-[var(--border-hairline)] pt-3 text-[12px]">
                  <div className="flex items-center justify-between">
                    <span className="text-[var(--ink-secondary)]">Status</span>
                    <span className="inline-flex items-center gap-1 font-medium text-[var(--ink-success)]">
                      <Check className="h-3.5 w-3.5" />
                      {selectedDay.state.record.flawless ? 'Flawless Solve' : 'Completed'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[var(--ink-secondary)]">Final Score</span>
                    <span className="tabular-nums font-semibold text-[var(--ink-primary)]">
                      {selectedDay.state.record.score.final_score.toLocaleString()}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[var(--ink-secondary)]">Solve Time</span>
                    <span className="tabular-nums font-medium text-[var(--ink-primary)]">
                      {formatSolveTime(selectedDay.state.record.solve_time_secs)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[var(--ink-secondary)]">Mistakes / Hints</span>
                    <span className="tabular-nums font-medium text-[var(--ink-primary)]">
                      {selectedDay.state.record.mistakes} / {selectedDay.state.record.hints_used}
                    </span>
                  </div>
                </div>
              ) : selectedDay.state.status === 'FutureLocked' ? (
                <div className="mt-4 rounded-lg bg-[var(--surface-canvas)] px-3 py-2.5 text-[12px] text-[var(--ink-secondary)]">
                  Unlocks on {formatDisplayDate(selectedDay.date)}.
                </div>
              ) : (
                <div className="mt-4">
                  <button
                    type="button"
                    disabled={meta.isBusy}
                    onClick={() => void actions.openDailyChallenge(formatIsoDate(selectedDay.date))}
                    className="inline-flex w-full items-center justify-center gap-1.5 rounded-lg bg-[var(--accent)] px-3.5 py-2 text-[13px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)] disabled:opacity-50"
                  >
                    <Play className="h-3.5 w-3.5 fill-current" />
                    {selectedDay.state.status === 'InProgress'
                      ? 'Resume Daily Challenge'
                      : 'Play Daily Challenge'}
                  </button>
                  <p className="mt-2 text-[11px] leading-snug text-[var(--ink-secondary)]">
                    Your active Standard Game slot stays safely paused while playing a Daily Challenge.
                  </p>
                </div>
              )}
            </div>
          ) : null}
        </aside>
      </div>
    </section>
  );
}
