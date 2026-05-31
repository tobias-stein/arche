import Phaser from 'phaser';
import { GAME_CONFIG } from '../config/game-config';
import { Dungeon } from './dungeon-generator';
import {
  generateRoomTiles,
  getPlayerSpawn,
  getDoorPos,
  oppositeDir,
  bfsPathfind,
  getDirFromDelta,
  TILE,
  type TileType,
} from './room-tiles';
import { drawRoom } from './room-renderer';
import { getGameState } from '../GameState';

const DIRS: [number, number][] = [[0, -1], [0, 1], [-1, 0], [1, 0]];

const KEY_MAP: Record<string, number> = {
  'w': 0, 'W': 0, 'ArrowUp': 0,
  's': 1, 'S': 1, 'ArrowDown': 1,
  'a': 2, 'A': 2, 'ArrowLeft': 2,
  'd': 3, 'D': 3, 'ArrowRight': 3,
};

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
  private playerGraphics!: Phaser.GameObjects.Graphics;
  private transitioning = false;
  private elapsed = 0;

  private moving = false;
  private moveFromX = 0;
  private moveFromY = 0;
  private moveToX = 0;
  private moveToY = 0;
  private moveStartTime = 0;

  private heldDirs: Set<number> = new Set();
  private moveCooldown = 0;
  private autoPath: { x: number; y: number }[] | null = null;

  private gameState = getGameState();

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
    this.playerGraphics = this.add.graphics();
    this.playerGraphics.setDepth(50);
    this.transitionOverlay = this.add.graphics();
    this.transitionOverlay.setDepth(100);

    this.generateDungeon();
    this.setupInput();
  }

  private setupInput(): void {
    this.input.keyboard!.on('keydown', (e: KeyboardEvent) => {
      const dir = KEY_MAP[e.key];
      if (dir !== undefined) {
        e.preventDefault();
        this.heldDirs.add(dir);
        this.autoPath = null;
        if (!this.moving && !this.transitioning) {
          this.tryMovePlayer(dir);
          this.moveCooldown = this.time.now + GAME_CONFIG.movement.holdMoveInterval;
        }
        return;
      }
    });

    this.input.keyboard!.on('keyup', (e: KeyboardEvent) => {
      const dir = KEY_MAP[e.key];
      if (dir !== undefined) {
        this.heldDirs.delete(dir);
      }
    });

    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      if (this.transitioning) return;

      const mx = pointer.x;
      const my = pointer.y;

      const tileX = Math.floor((mx - this.offsetX) / this.tileSize);
      const tileY = Math.floor((my - this.offsetY) / this.tileSize);

      if (tileX < 0 || tileX >= GAME_CONFIG.room.width || tileY < 0 || tileY >= GAME_CONFIG.room.height) return;
      if (tileX === this.playerX && tileY === this.playerY) return;

      const tile = this.tiles[tileY][tileX];
      if (tile === TILE.WALL) return;

      const path = bfsPathfind(this.tiles, this.playerX, this.playerY, tileX, tileY);
      if (path) {
        this.autoPath = path;
        this.heldDirs.clear();
      }
    });
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
    this.transitioning = false;
    this.moving = false;
    this.autoPath = null;
    this.heldDirs.clear();
    this.gameState.setDungeonData(this.dungeon);
    this.gameState.setCurrentRoom(this.currentRoom);
    this.gameState.setPlayerPosition(this.playerX, this.playerY);
    this.drawCurrentRoom();
  }

  private drawCurrentRoom(): void {
    this.roomGraphics.clear();
    this.vignetteGraphics.clear();
    this.playerGraphics.clear();

    drawRoom(this.roomGraphics, this.tiles, this.tileSize, this.offsetX, this.offsetY, this.elapsed);

    this.drawPlayer();

    this.drawVignette();
  }

  private drawPlayer(): void {
    const px = this.offsetX + this.renderX * this.tileSize;
    const py = this.offsetY + this.renderY * this.tileSize;
    const cx = Math.round(px + this.tileSize / 2);
    const cy = Math.round(py + this.tileSize / 2);
    const margin = Math.round(this.tileSize * 0.16);
    const r = Math.round(this.tileSize * 0.5 - margin);

    this.playerGraphics.fillStyle(0xe8dfd0, 1);
    this.playerGraphics.fillCircle(cx, cy, r);
    this.playerGraphics.fillStyle(0xd0c8b8, 1);
    this.playerGraphics.fillCircle(cx, cy, Math.max(1, r - Math.round(this.tileSize * 0.04)));

    this.playerGraphics.lineStyle(Math.max(1, Math.round(this.tileSize * 0.025)), 0xb0a898, 1);
    this.playerGraphics.strokeCircle(cx, cy, r);
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

  private tryMovePlayer(dir: number): void {
    if (this.transitioning || this.moving) return;

    const nx = this.playerX + DIRS[dir][0];
    const ny = this.playerY + DIRS[dir][1];
    if (nx < 0 || nx >= GAME_CONFIG.room.width || ny < 0 || ny >= GAME_CONFIG.room.height) return;

    const tile = this.tiles[ny][nx];
    if (tile === TILE.WALL) return;

    if (tile === TILE.DOOR_N || tile === TILE.DOOR_S || tile === TILE.DOOR_E || tile === TILE.DOOR_W) {
      const nextRoom = this.dungeon.getNeighbor(this.currentRoom, dir);
      if (nextRoom !== -1) {
        this.transitionToRoom(nextRoom, dir);
      }
      return;
    }

    this.startMove(nx, ny);
  }

  private startMove(tx: number, ty: number): void {
    this.moving = true;
    this.moveFromX = this.playerX;
    this.moveFromY = this.playerY;
    this.moveToX = tx;
    this.moveToY = ty;
    this.moveStartTime = this.time.now;
    this.playerX = tx;
    this.playerY = ty;
    this.gameState.setPlayerPosition(tx, ty);
  }

  transitionToRoom(nextRoom: number, enterDir: number): void {
    if (this.transitioning) return;
    this.transitioning = true;
    this.autoPath = null;

    const duration = GAME_CONFIG.dungeon.transitionMs;
    const half = duration / 2;

    const fadeTarget = { alpha: 0 };
    this.tweens.add({
      targets: fadeTarget,
      alpha: 1,
      duration: half,
      onUpdate: () => this.drawTransitionOverlay(fadeTarget.alpha),
      onComplete: () => {
        this.currentRoom = nextRoom;
        this.tiles = generateRoomTiles(this.dungeon, this.currentRoom);
        this.gameState.setCurrentRoom(nextRoom);

        const pos = getDoorPos(oppositeDir(enterDir));
        this.playerX = pos.x;
        this.playerY = pos.y;
        this.renderX = this.playerX;
        this.renderY = this.playerY;
        this.gameState.setPlayerPosition(pos.x, pos.y);

        const fadeOutTarget = { alpha: 1 };
        this.tweens.add({
          targets: fadeOutTarget,
          alpha: 0,
          duration: half,
          onUpdate: () => this.drawTransitionOverlay(fadeOutTarget.alpha),
          onComplete: () => {
            this.transitionOverlay.clear();
            this.transitioning = false;
          },
        });
      },
    });
  }

  private drawTransitionOverlay(alpha: number): void {
    this.transitionOverlay.clear();
    this.transitionOverlay.fillStyle(0x000000, alpha);
    this.transitionOverlay.fillRect(0, 0, this.scale.width, this.scale.height);
  }

  private processInput(): void {
    if (this.autoPath && this.autoPath.length > 0 && !this.moving && !this.transitioning) {
      const next = this.autoPath[0];
      const dir = getDirFromDelta(next.x - this.playerX, next.y - this.playerY);
      if (dir !== -1) {
        const tile = this.tiles[next.y][next.x];
        if (tile === TILE.DOOR_N || tile === TILE.DOOR_S || tile === TILE.DOOR_E || tile === TILE.DOOR_W) {
          this.autoPath = null;
          this.tryMovePlayer(dir);
          return;
        }
        this.tryMovePlayer(dir);
        if (this.moving) {
          this.autoPath.shift();
        }
      } else {
        this.autoPath = null;
      }
      return;
    }

    if (this.heldDirs.size > 0 && !this.moving && !this.transitioning) {
      if (this.time.now >= this.moveCooldown) {
        const dir = [...this.heldDirs][this.heldDirs.size - 1];
        this.autoPath = null;
        this.tryMovePlayer(dir);
        this.moveCooldown = this.time.now + GAME_CONFIG.movement.holdMoveInterval;
      }
    }
  }

  private updateMovement(): void {
    if (!this.moving) return;

    const elapsed = this.time.now - this.moveStartTime;
    const t = Math.min(1, elapsed / GAME_CONFIG.movement.walkSpeed);

    this.renderX = this.lerp(this.moveFromX, this.moveToX, t);
    this.renderY = this.lerp(this.moveFromY, this.moveToY, t);

    if (t >= 1) {
      this.renderX = this.moveToX;
      this.renderY = this.moveToY;
      this.moving = false;
    }
  }

  private lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
  }

  update(_time: number, delta: number): void {
    this.elapsed += delta;
    this.processInput();
    this.updateMovement();
    this.drawCurrentRoom();
  }
}
