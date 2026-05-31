import Phaser from 'phaser';
import type { Difficulty } from '../types';

export const DIFFICULTY_COLORS: Record<Difficulty, number> = {
  normal: 0x889096,
  champion: 0xd4883a,
  elite: 0xc83838,
  boss: 0x7a388a,
};

export const SYMBOL_COLOR = 0xf0e8d8;

function lighten(color: number, amount: number): number {
  const r = Math.min(255, ((color >> 16) & 0xff) + amount);
  const g = Math.min(255, ((color >> 8) & 0xff) + amount);
  const b = Math.min(255, (color & 0xff) + amount);
  return (r << 16) | (g << 8) | b;
}

function darken(color: number, amount: number): number {
  const r = Math.max(0, ((color >> 16) & 0xff) - amount);
  const g = Math.max(0, ((color >> 8) & 0xff) - amount);
  const b = Math.max(0, (color & 0xff) - amount);
  return (r << 16) | (g << 8) | b;
}

export function drawCreatureTriangle(
  graphics: Phaser.GameObjects.Graphics,
  px: number,
  py: number,
  ts: number,
  difficulty: Difficulty,
): void {
  const color = DIFFICULTY_COLORS[difficulty];
  const cx = Math.round(px + ts / 2);
  const cy = Math.round(py + ts / 2);
  const margin = Math.round(ts * 0.20);
  const topY = Math.round(py + margin);
  const botY = Math.round(py + ts - margin);
  const height = botY - topY;
  const halfW = Math.round(height * 0.577);

  const lighter = lighten(color, 50);
  const darker = darken(color, 40);

  const lr = (lighter >> 16) & 0xff;
  const lg = (lighter >> 8) & 0xff;
  const lb = lighter & 0xff;

  const dr = (darker >> 16) & 0xff;
  const dg = (darker >> 8) & 0xff;
  const db = darker & 0xff;

  const cr = Math.max(2, Math.round(ts * 0.045));

  graphics.fillStyle(Phaser.Display.Color.GetColor(lr, lg, lb));
  graphics.fillTriangle(cx, topY, cx + halfW, botY, cx - halfW, botY);

  graphics.lineStyle(cr * 2, color);
  graphics.strokeTriangle(cx, topY, cx + halfW, botY, cx - halfW, botY);

  graphics.lineStyle(Math.max(1, Math.round(ts * 0.025)), Phaser.Display.Color.GetColor(dr, dg, db));
  graphics.strokeTriangle(cx, topY, cx + halfW, botY, cx - halfW, botY);
}

export function createCreatureLabel(
  scene: Phaser.Scene,
  cx: number,
  cy: number,
  ts: number,
): Phaser.GameObjects.Text {
  const fontSize = Math.round(ts * 0.3);
  return scene.add.text(cx, Math.round(cy + ts * 0.06), '!', {
    fontFamily: 'sans-serif',
    fontSize: `${fontSize}px`,
    color: '#f0e8d8',
    fontStyle: 'bold',
  }).setOrigin(0.5);
}
