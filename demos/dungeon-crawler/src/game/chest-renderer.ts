import Phaser from 'phaser';

const CHEST_COLOR = 0x8b7355;
const CHEST_LIGHT = 0xa99173;
const SYM_COLOR = '#f0e8d8';
const CHEST_OPEN = 0x6a5a40;
const CHEST_OPEN_LIGHT = 0x8a7a60;

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

export function drawChest(
  graphics: Phaser.GameObjects.Graphics,
  px: number,
  py: number,
  ts: number,
  highlight: boolean,
): void {
  const margin = Math.round(ts * 0.16);
  const rr = Math.max(1, Math.round(ts * 0.06));
  const w = ts - margin * 2;
  const h = ts - margin * 2;
  const x = px + margin;
  const y = py + margin;

  if (highlight) {
    graphics.fillStyle(lighten(CHEST_LIGHT, 40), 0.3);
    graphics.fillRoundedRect(px + 2, py + 2, ts - 4, ts - 4, rr + 2);
  }

  graphics.fillGradientStyle(CHEST_LIGHT, CHEST_LIGHT, CHEST_COLOR, CHEST_COLOR);
  graphics.fillRoundedRect(x, y, w, h, rr);

  graphics.lineStyle(Math.max(1, Math.round(ts * 0.025)), darken(CHEST_COLOR, 40), 1);
  graphics.strokeRoundedRect(x, y, w, h, rr);
}

export function drawChestText(
  scene: Phaser.Scene,
  px: number,
  py: number,
  ts: number,
): Phaser.GameObjects.Text {
  const cx = Math.round(px + ts / 2);
  const fontSize = Math.round(ts * 0.34);
  return scene.add.text(cx, Math.round(py + ts / 2 + ts * 0.01), '?', {
    fontFamily: 'sans-serif',
    fontSize: `${fontSize}px`,
    color: SYM_COLOR,
    fontStyle: 'bold',
  }).setOrigin(0.5).setDepth(35);
}

export function drawOpenChest(
  graphics: Phaser.GameObjects.Graphics,
  px: number,
  py: number,
  ts: number,
): void {
  const margin = Math.round(ts * 0.16);
  const rr = Math.max(1, Math.round(ts * 0.06));
  const w = ts - margin * 2;
  const h = ts - margin * 2;
  const x = px + margin;
  const y = py + margin;

  graphics.fillGradientStyle(CHEST_OPEN_LIGHT, CHEST_OPEN_LIGHT, CHEST_OPEN, CHEST_OPEN);
  graphics.fillRoundedRect(x, y, w, h, rr);

  graphics.lineStyle(Math.max(1, Math.round(ts * 0.025)), darken(CHEST_OPEN, 40), 1);
  graphics.strokeRoundedRect(x, y, w, h, rr);
}
