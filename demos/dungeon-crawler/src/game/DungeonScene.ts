import Phaser from 'phaser';
import { GAME_CONFIG } from '../config/game-config';
import { Dungeon } from './dungeon-generator';
import {
  generateRoomTiles,
  getPlayerSpawn,
  getDoorPos,
  oppositeDir,
  TileType,
} from './room-tiles';
import { drawRoom } from './room-renderer';

export class DungeonScene extends Phaser.Scene {
  private dungeon!: Dungeon;
  private currentRoom = 0;
  private tiles!: TileType[][];
  private playerX = 0;
  private playerY = 0;
  private renderX = 0;
  private renderY = 0;
  private tileSize = 0;
  private offsetX = 0;
  private offsetY = 0;
  private roomGraphics!: Phaser.GameObjects.Graphics;
  private vignetteGraphics!: Phaser.GameObjects.Graphics;
  private transitionOverlay!: Phaser.GameObjects.Graphics;
  private transitioning = false;
  private visited: Set<number> = new Set();
  private elapsed = 0;

  constructor() {
    super({ key: 'DungeonScene' });
  }

  create(): void {
    this.tileSize = Math.min(
      Math.floor(this.scale.width / GAME_CONFIG.room.width),
      Math.floor(this.scale.height / GAME_CONFIG.room.height),
    );
    this.offsetX = Math.floor((this.scale.width - GAME_CONFIG.room.width * this.tileSize) / 2);
    this.offsetY = Math.floor((this.scale.height - GAME_CONFIG.room.height * this.tileSize) / 2);

    this.roomGraphics = this.add.graphics();
    this.vignetteGraphics = this.add.graphics();
    this.transitionOverlay = this.add.graphics();
    this.transitionOverlay.setDepth(100);

    this.generateDungeon();
  }

  generateDungeon(): void {
    this.dungeon = new Dungeon();
    this.currentRoom = this.dungeon.entranceRoom;
    this.tiles = generateRoomTiles(this.dungeon, this.currentRoom);
    const spawn = getPlayerSpawn();
    this.playerX = spawn.x;
    this.playerY = spawn.y;
    this.renderX = this.playerX;
    this.renderY = this.playerY;
    this.visited = new Set([this.currentRoom]);
    this.transitioning = false;
    this.drawCurrentRoom();
  }

  private drawCurrentRoom(): void {
    this.roomGraphics.clear();
    this.vignetteGraphics.clear();

    drawRoom(this.roomGraphics, this.tiles, this.tileSize, this.offsetX, this.offsetY, this.elapsed);

    this.drawPlayer();

    this.drawVignette();
  }

  private drawPlayer(): void {
    const px = this.offsetX + this.renderX * this.tileSize;
    const py = this.offsetY + this.renderY * this.tileSize;
    const pad = Math.floor(this.tileSize * 0.15);
    const size = this.tileSize - pad * 2;
    const cx = px + pad + size / 2;
    const cy = py + pad + size / 2;
    const r = size / 2;

    this.roomGraphics.fillStyle(0x4fc3f7, 1);
    this.roomGraphics.fillCircle(cx, cy, r);

    const inner = Math.floor(this.tileSize * 0.25);
    this.roomGraphics.fillStyle(0x81d4fa, 1);
    this.roomGraphics.fillCircle(cx, cy, r - inner / 2);
  }

  private drawVignette(): void {
    const w = this.scale.width;
    const h = this.scale.height;
    const grd = this.vignetteGraphics;
    grd.fillStyle(0x000000, 0);
    const innerR = Math.min(w, h) * 0.35;

    const totalSteps = 20;
    for (let i = 0; i < totalSteps; i++) {
      const t = i / totalSteps;
      const r2 = innerR + (Math.max(w, h) * 0.65 - innerR) * (t + 1 / totalSteps);
      const alpha = t * t * 0.45;
      grd.fillStyle(0x000000, alpha);
      grd.fillCircle(w / 2, h / 2, r2);
    }
  }

  transitionToRoom(nextRoom: number, enterDir: number): void {
    if (this.transitioning) return;
    this.transitioning = true;

    const overlay = this.transitionOverlay;
    const duration = GAME_CONFIG.dungeon.transitionMs;
    const half = duration / 2;

    const fadeTarget = { alpha: 0 };
    this.tweens.add({
      targets: fadeTarget,
      alpha: 1,
      duration: half,
      onUpdate: () => {
        overlay.clear();
        overlay.fillStyle(0x000000, fadeTarget.alpha);
        overlay.fillRect(0, 0, this.scale.width, this.scale.height);
      },
      onComplete: () => {
        this.currentRoom = nextRoom;
        this.tiles = generateRoomTiles(this.dungeon, this.currentRoom);
        this.visited.add(nextRoom);

        const pos = getDoorPos(oppositeDir(enterDir));
        this.playerX = pos.x;
        this.playerY = pos.y;
        this.renderX = this.playerX;
        this.renderY = this.playerY;

        const fadeOutTarget = { alpha: 1 };
        this.tweens.add({
          targets: fadeOutTarget,
          alpha: 0,
          duration: half,
          onUpdate: () => {
            overlay.clear();
            overlay.fillStyle(0x000000, fadeOutTarget.alpha);
            overlay.fillRect(0, 0, this.scale.width, this.scale.height);
          },
          onComplete: () => {
            overlay.clear();
            this.transitioning = false;
          },
        });
      },
    });
  }

  update(_time: number, delta: number): void {
    this.elapsed += delta;
    this.drawCurrentRoom();
  }
}
