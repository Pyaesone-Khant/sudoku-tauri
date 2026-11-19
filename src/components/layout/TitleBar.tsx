import { useEffect, useRef, useState } from 'react';
import { Monitor, Moon, SlidersHorizontal, Sun, Volume2, VolumeX } from 'lucide-react';
import { useGame, type AppTab } from '../../context/GameProvider';
import { usePreferences, type ThemePreference } from '../../context/PreferencesProvider';

const TABS: readonly { id: AppTab; label: string }[] = [
  { id: 'play', label: 'Play' },
  { id: 'daily', label: 'Daily' },
  { id: 'stats', label: 'Stats' },
];

const THEME_OPTIONS: readonly { id: ThemePreference; label: string; icon: typeof Sun }[] = [
  { id: 'system', label: 'System', icon: Monitor },
  { id: 'light', label: 'Light', icon: Sun },
  { id: 'dark', label: 'Dark', icon: Moon },
];

/**
 * macOS-native overlay titlebar with drag region, segmented navigation, and Settings popover.
 */
export function TitleBar() {
  const { state, actions } = useGame();
  const { theme, muted, setTheme, toggleMuted } = usePreferences();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!settingsOpen) {
      return;
    }
    function handleOutside(event: MouseEvent) {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        setSettingsOpen(false);
      }
    }
    window.addEventListener('mousedown', handleOutside);
    return () => window.removeEventListener('mousedown', handleOutside);
  }, [settingsOpen]);

  return (
    <header
      data-tauri-drag-region
      className="relative flex h-12 shrink-0 items-center justify-between border-b border-[var(--border-hairline)] bg-[var(--surface-elevated)]/85 px-4 pl-20 backdrop-blur-md"
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <span className="text-[13px] font-semibold tracking-tight text-[var(--ink-primary)]">
          Sudoku
        </span>
      </div>

      <nav
        aria-label="Primary navigation"
        className="flex items-center rounded-lg bg-[var(--surface-subtle)] p-0.5"
      >
        {TABS.map((tab) => {
          const active = state.activeTab === tab.id;
          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => actions.setActiveTab(tab.id)}
              aria-current={active ? 'page' : undefined}
              className={`rounded-md px-3.5 py-1 text-[12px] font-medium transition-colors ${
                active
                  ? 'bg-[var(--surface-elevated)] text-[var(--ink-primary)] shadow-xs'
                  : 'text-[var(--ink-secondary)] hover:text-[var(--ink-primary)]'
              }`}
            >
              {tab.label}
            </button>
          );
        })}
      </nav>

      <div ref={popoverRef} className="relative flex items-center gap-1.5">
        <button
          type="button"
          onClick={toggleMuted}
          aria-label={muted ? 'Unmute sound effects' : 'Mute sound effects'}
          title={muted ? 'Sound muted' : 'Sound on'}
          className="flex h-7 w-7 items-center justify-center rounded-md text-[var(--ink-secondary)] transition-colors hover:bg-[var(--surface-subtle)] hover:text-[var(--ink-primary)]"
        >
          {muted ? <VolumeX className="h-3.5 w-3.5" /> : <Volume2 className="h-3.5 w-3.5" />}
        </button>

        <button
          type="button"
          onClick={() => setSettingsOpen((open) => !open)}
          aria-label="Appearance settings"
          aria-expanded={settingsOpen}
          className={`flex h-7 w-7 items-center justify-center rounded-md transition-colors ${
            settingsOpen
              ? 'bg-[var(--surface-subtle)] text-[var(--ink-primary)]'
              : 'text-[var(--ink-secondary)] hover:bg-[var(--surface-subtle)] hover:text-[var(--ink-primary)]'
          }`}
        >
          <SlidersHorizontal className="h-3.5 w-3.5" />
        </button>

        {settingsOpen ? (
          <div
            role="dialog"
            aria-label="Settings"
            className="absolute right-0 top-9 z-40 w-56 rounded-xl border border-[var(--border-hairline)] bg-[var(--surface-elevated)] p-3 shadow-lg"
          >
            <div className="mb-2 text-[12px] font-medium text-[var(--ink-secondary)]">
              Appearance
            </div>
            <div className="grid grid-cols-3 gap-1 rounded-lg bg-[var(--surface-subtle)] p-0.5">
              {THEME_OPTIONS.map((opt) => {
                const Icon = opt.icon;
                const selected = theme === opt.id;
                return (
                  <button
                    key={opt.id}
                    type="button"
                    onClick={() => setTheme(opt.id)}
                    className={`flex flex-col items-center gap-1 rounded-md py-1.5 text-[11px] font-medium transition-colors ${
                      selected
                        ? 'bg-[var(--surface-elevated)] text-[var(--ink-primary)] shadow-xs'
                        : 'text-[var(--ink-secondary)] hover:text-[var(--ink-primary)]'
                    }`}
                  >
                    <Icon className="h-3.5 w-3.5" />
                    {opt.label}
                  </button>
                );
              })}
            </div>

            <div className="mt-3 flex items-center justify-between border-t border-[var(--border-hairline)] pt-2.5">
              <span className="text-[12px] text-[var(--ink-secondary)]">Sound cues</span>
              <button
                type="button"
                onClick={toggleMuted}
                className="rounded-md bg-[var(--surface-subtle)] px-2.5 py-1 text-[11px] font-medium text-[var(--ink-primary)] hover:bg-[var(--surface-control)]"
              >
                {muted ? 'Muted' : 'On'}
              </button>
            </div>
          </div>
        ) : null}
      </div>
    </header>
  );
}
