import Phaser from 'phaser';
import { GAME_CONFIG } from '../config/game-config';
import { TILE, type TileType } from './room-tiles';

const PALETTE = {
  wallBase: 0x2d4d38,
  wallHi: 0x4a7a58,
  wallLo: 0x1a3a2a,
  wallDark: 0x1a2e22,
  wallBrick: 0x3a5a44,
  floorA: 0x3a6a48,
  floorB: 0x2a4a38,
  tunnelBg: 0x0a1a10,
};

export function drawWallTile(graphics: Phaser.GameObjects.Graphics, px: number, py: number, ts: number): void {
  graphics.fillStyle(PALETTE.wallBase);
  graphics.fillRect(px, py, ts, ts);

  graphics.fillStyle(PALETTE.wallHi);
  graphics.fillRect(px, py, ts, 2);
  graphics.fillRect(px, py, 2, ts);

  graphics.fillStyle(PALETTE.wallLo);
  graphics.fillRect(px + ts - 2, py, 2, ts);
  graphics.fillRect(px, py + ts - 2, ts, 2);

  const row = Math.floor(py / ts);
  if (row % 2 === 0) {
    graphics.fillStyle(PALETTE.wallBrick);
    graphics.fillRect(px + 2, py + 3, ts - 8, 1);
  } else {
    graphics.fillStyle(PALETTE.wallBrick);
    graphics.fillRect(px + 6, py + 3, ts - 10, 1);
  }

  const col = Math.floor(px / ts);
  if (col % 3 === 1) {
    graphics.fillStyle(PALETTE.wallDark);
    graphics.fillRect(px + 5, py + 2, 1, ts - 4);
  }
}

export function drawArchway(graphics: Phaser.GameObjects.Graphics, px: number, py: number, ts: number, dir: string): void {
  drawWallTile(graphics, px, py, ts);

  const pad = Math.max(2, Math.floor(ts * 0.04));
  const cx = px + ts * 0.5;
  const cy = py + ts * 0.5;
  const split = 0.35;

  graphics.fillStyle(PALETTE.tunnelBg);
  graphics.beginPath();

  if (dir === 'N') {
    const s = py + ts * split;
    graphics.moveTo(px + pad, py + ts - pad);
    graphics.lineTo(px + pad, s);
    graphics.lineTo(cx, py + pad);
    graphics.lineTo(px + ts - pad, s);
    graphics.lineTo(px + ts - pad, py + ts - pad);
  } else if (dir === 'S') {
    const s = py + ts * (1 - split);
    graphics.moveTo(px + pad, py + pad);
    graphics.lineTo(px + pad, s);
    graphics.lineTo(cx, py + ts - pad);
    graphics.lineTo(px + ts - pad, s);
    graphics.lineTo(px + ts - pad, py + pad);
  } else if (dir === 'W') {
    const s = px + ts * split;
    graphics.moveTo(px + ts - pad, py + pad);
    graphics.lineTo(s, py + pad);
    graphics.lineTo(px + pad, cy);
    graphics.lineTo(s, py + ts - pad);
    graphics.lineTo(px + ts - pad, py + ts - pad);
  } else if (dir === 'E') {
    const s = px + ts * (1 - split);
    graphics.moveTo(px + pad, py + pad);
    graphics.lineTo(s, py + pad);
    graphics.lineTo(px + ts - pad, cy);
    graphics.lineTo(s, py + ts - pad);
    graphics.lineTo(px + pad, py + ts - pad);
  }

  graphics.closePath();
  graphics.fill();
}

const DOOR_DIR: Record<number, string> = {
  [TILE.DOOR_N]: 'N',
  [TILE.DOOR_S]: 'S',
  [TILE.DOOR_E]: 'E',
  [TILE.DOOR_W]: 'W',
};

export function drawDoorGlow(graphics: Phaser.GameObjects.Graphics, px: number, py: number, ts: number, time: number, dir: string): void {
  const glow = 0.5 + 0.5 * Math.sin(time * 0.003);
  const g = Math.floor(lerp(80, 200, glow));
  const isNS = dir === 'N' || dir === 'S';

  graphics.fillStyle(Phaser.Display.Color.GetColor(g + 40, g, 0));
  graphics.fillRect(px + 4, py + (isNS ? 2 : 0), ts - 8, ts - (isNS ? 4 : 0));

  graphics.fillStyle(Phaser.Display.Color.GetColor(Math.min(255, g + 80), Math.min(255, g + 40), 40));
  graphics.fillRect(px + 8, py + 4, ts - 16, ts - 8);

  if (glow > 0.7) {
    graphics.fillStyle(Phaser.Display.Color.GetColor(255, 220, 100));
    graphics.fillRect(px + 4 + Math.floor(Math.random() * (ts - 8)), py + 4 + Math.floor(Math.random() * (ts - 8)), 3, 3);
  }
}

export function drawRoom(
  graphics: Phaser.GameObjects.Graphics,
  tiles: TileType[][],
  tileSize: number,
  offsetX: number,
  offsetY: number,
  time: number,
): void {
  const rw = GAME_CONFIG.room.width;
  const rh = GAME_CONFIG.room.height;

  for (let y = 0; y < rh; y++) {
    for (let x = 0; x < rw; x++) {
      const px = offsetX + x * tileSize;
      const py = offsetY + y * tileSize;
      const t = tiles[y][x];

      switch (t) {
        case TILE.WALL:
          drawWallTile(graphics, px, py, tileSize);
          break;
        case TILE.FLOOR: {
          const isLight = (x + y) % 2 === 0;
          graphics.fillStyle(isLight ? PALETTE.floorA : PALETTE.floorB);
          graphics.fillRect(px, py, tileSize, tileSize);
          graphics.fillStyle(0x000000, 0.12);
          graphics.fillRect(px, py, tileSize, 1);
          graphics.fillRect(px, py, 1, tileSize);
          break;
        }
        case TILE.DOOR_N:
        case TILE.DOOR_S:
        case TILE.DOOR_E:
        case TILE.DOOR_W:
          drawArchway(graphics, px, py, tileSize, DOOR_DIR[t]);
          drawDoorGlow(graphics, px, py, tileSize, time, DOOR_DIR[t]);
          break;
      }
    }
  }
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}
