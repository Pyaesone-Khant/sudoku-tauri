import { useEffect, useRef } from 'react';

interface ConfettiCanvasProps {
  /** Triggers a fresh celebration burst whenever `active` is true for a new `burstKey`. */
  active: boolean;
  burstKey: number | string;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  aspect: number;
  color: string;
  rotation: number;
  spin: number;
  life: number;
  maxLife: number;
}

const PALETTE = ['#0066cc', '#5e9eff', '#34c759', '#f5b942', '#ff6961', '#af52de'] as const;

/**
 * Lightweight zero-dependency HTML5 canvas confetti overlay for Puzzle completion.
 * Automatically stops animating once settled and honors `prefers-reduced-motion`.
 */
export function ConfettiCanvas({ active, burstKey }: ConfettiCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!active) {
      return;
    }
    if (
      typeof window !== 'undefined' &&
      window.matchMedia &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches
    ) {
      return;
    }

    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    const width = window.innerWidth;
    const height = window.innerHeight;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    ctx.scale(dpr, dpr);

    // Spawn two balanced side-arc bursts so confetti frames the Victory card cleanly.
    const particles: Particle[] = [];
    const perSide = 44;
    for (let side = 0; side < 2; side += 1) {
      const originX = side === 0 ? width * 0.18 : width * 0.82;
      const originY = height * 0.62;
      const baseAngle = side === 0 ? -Math.PI / 3 : (-2 * Math.PI) / 3;

      for (let i = 0; i < perSide; i += 1) {
        const spread = (Math.random() - 0.5) * 0.95;
        const speed = 6.5 + Math.random() * 8.5;
        const angle = baseAngle + spread;
        particles.push({
          x: originX,
          y: originY,
          vx: Math.cos(angle) * speed,
          vy: Math.sin(angle) * speed - 2.2,
          size: 5 + Math.random() * 4,
          aspect: 0.45 + Math.random() * 0.55,
          color: PALETTE[i % PALETTE.length] ?? '#0066cc',
          rotation: Math.random() * Math.PI * 2,
          spin: (Math.random() - 0.5) * 0.24,
          life: 0,
          maxLife: 110 + Math.random() * 55,
        });
      }
    }

    let frameId = 0;
    const tick = () => {
      ctx.clearRect(0, 0, width, height);
      let alive = 0;

      for (const p of particles) {
        p.life += 1;
        if (p.life >= p.maxLife) {
          continue;
        }
        alive += 1;
        p.vx *= 0.985;
        p.vy = p.vy * 0.985 + 0.22;
        p.x += p.vx;
        p.y += p.vy;
        p.rotation += p.spin;

        const fade = Math.max(0, 1 - Math.max(0, p.life - p.maxLife * 0.65) / (p.maxLife * 0.35));
        ctx.save();
        ctx.globalAlpha = fade;
        ctx.translate(p.x, p.y);
        ctx.rotate(p.rotation);
        ctx.fillStyle = p.color;
        ctx.fillRect(-p.size / 2, -(p.size * p.aspect) / 2, p.size, p.size * p.aspect);
        ctx.restore();
      }

      if (alive > 0) {
        frameId = window.requestAnimationFrame(tick);
      } else {
        ctx.clearRect(0, 0, width, height);
      }
    };

    frameId = window.requestAnimationFrame(tick);
    return () => {
      window.cancelAnimationFrame(frameId);
    };
  }, [active, burstKey]);

  if (!active) {
    return null;
  }

  return (
    <canvas
      ref={canvasRef}
      aria-hidden="true"
      data-testid="victory-confetti"
      className="pointer-events-none fixed inset-0 z-50 h-full w-full"
    />
  );
}
