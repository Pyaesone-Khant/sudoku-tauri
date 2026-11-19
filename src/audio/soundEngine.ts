/**
 * Zero-asset Web Audio API synthesizer for subtle macOS-style tactile cues.
 * Adds 0 KB of bundled media assets.
 */

export type SoundCue = 'place' | 'note' | 'mistake' | 'hint' | 'victory';

let audioContext: AudioContext | null = null;

function getAudioContext(): AudioContext | null {
  if (typeof window === 'undefined') {
    return null;
  }
  const Ctor = window.AudioContext;
  if (!Ctor) {
    return null;
  }
  if (!audioContext) {
    audioContext = new Ctor();
  }
  if (audioContext.state === 'suspended') {
    void audioContext.resume();
  }
  return audioContext;
}

function playTone(
  ctx: AudioContext,
  freq: number,
  durationSec: number,
  gainPeak: number,
  type: OscillatorType,
  startOffsetSec = 0,
): void {
  const now = ctx.currentTime + startOffsetSec;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, now);

  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.exponentialRampToValueAtTime(gainPeak, now + 0.008);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + durationSec);

  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + durationSec + 0.01);
}

/**
 * Plays a synthesized UI sound cue unless muted.
 */
export function playSoundCue(cue: SoundCue, muted: boolean): void {
  if (muted) {
    return;
  }
  const ctx = getAudioContext();
  if (!ctx) {
    return;
  }

  switch (cue) {
    case 'place':
      playTone(ctx, 520, 0.06, 0.055, 'sine');
      break;
    case 'note':
      playTone(ctx, 740, 0.035, 0.03, 'triangle');
      break;
    case 'mistake':
      playTone(ctx, 185, 0.14, 0.08, 'triangle');
      break;
    case 'hint':
      playTone(ctx, 660, 0.09, 0.045, 'sine', 0);
      playTone(ctx, 880, 0.12, 0.045, 'sine', 0.07);
      break;
    case 'victory':
      playTone(ctx, 523.25, 0.16, 0.06, 'sine', 0);
      playTone(ctx, 659.25, 0.16, 0.06, 'sine', 0.1);
      playTone(ctx, 783.99, 0.2, 0.065, 'sine', 0.2);
      playTone(ctx, 1046.5, 0.35, 0.07, 'sine', 0.31);
      break;
  }
}
