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
  getAllDoorPositions,
  getTileRegion,
  TILE,
  type TileType,
} from './room-tiles';
import { drawRoom } from './room-renderer';
import { getGameState } from '../GameState';
import {
  generateCreaturesForRoom,
} from './creature-spawner';
import { generateChestsForRoom } from './chest-spawner';
import { drawCreatureTriangle, createCreatureLabel } from './creature-renderer';
import { drawChest, drawOpenChest, drawChestText } from './chest-renderer';
import type { CreatureState, ChestState } from '../types';

const DIRS: [number, number][] = [[0, -1], [0, 1], [-1, 0], [1, 0]];

const KEY_MAP: Record<string, number> = {
  'w': 0, 'W': 0, 'ArrowUp': 0,
  's': 1, 'S': 1, 'ArrowDown': 1,
  'a': 2, 'A': 2, 'ArrowLeft': 2,
  'd': 3, 'D': 3, 'ArrowRight': 3,
};

export class DungeonScene extends Phaser.Scene {
  static currentInstance: DungeonScene | null = null

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
  private creatureGraphics!: Phaser.GameObjects.Graphics;
  private chestGraphics!: Phaser.GameObjects.Graphics;
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

  private lastMoveFromX = 0;
  private lastMoveFromY = 0;

  private creatures: CreatureState[] = [];
  private creatureLabels: Phaser.GameObjects.Text[] = [];
  private chaseTarget: CreatureState | null = null;
  private chests: ChestState[] = [];
  private chestTexts: (Phaser.GameObjects.Text | null)[] = [];
  private adjacentChest: ChestState | null = null;
  private chaseTickAccum = 0;

  constructor() {
    super({ key: 'DungeonScene' });
  }

  private boundHandleVictory: ((creatureId: string) => void) | null = null
  private boundHandleFled: ((creatureId: string) => void) | null = null
  private boundHandleBackAway: ((creatureId: string) => void) | null = null

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
    this.creatureGraphics = this.add.graphics();
    this.creatureGraphics.setDepth(40);
    this.chestGraphics = this.add.graphics();
    this.chestGraphics.setDepth(30);
    this.transitionOverlay = this.add.graphics();
    this.transitionOverlay.setDepth(100);

    this.generateDungeon();
    this.setupInput();

    this.boundHandleVictory = (creatureId: string) => this.removeCreature(creatureId)
    this.boundHandleFled = (creatureId: string) => this.stunCreature(creatureId)
    this.boundHandleBackAway = (_creatureId: string) => this.movePlayerBack()
    this.gameState.on('combat:victory', this.boundHandleVictory)
    this.gameState.on('combat:fled', this.boundHandleFled)
    this.gameState.on('encounter:backed-away', this.boundHandleBackAway)

    DungeonScene.currentInstance = this
  }

  private setupInput(): void {
    this.input.keyboard!.on('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        if (!this.transitioning && !this.moving) {
          this.interactWithAdjacentChest();
        }
        return;
      }

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
    void this.spawnCreatures();
    this.drawCurrentRoom();
  }

  private async spawnCreatures(): Promise<void> {
    this.destroyCreatureLabels();
    this.destroyChestTexts();

    const isBoss = this.currentRoom === this.dungeon.bossRoom;
    const playerLevel = this.gameState.player.level;

    // Exclude floor tiles within 1 tile of any door
    const excludedPositions = getAllDoorPositions(this.tiles)
      .flatMap(door => getTileRegion(this.tiles, door, 1));

    this.creatures = await generateCreaturesForRoom(
      this.tiles,
      this.currentRoom,
      isBoss,
      playerLevel,
      excludedPositions,
    );

    this.creatureLabels = this.creatures.map((c) => {
      const px = this.offsetX + c.position.x * this.tileSize;
      const py = this.offsetY + c.position.y * this.tileSize;
      const cx = Math.round(px + this.tileSize / 2);
      const cy = Math.round(py + this.tileSize / 2);
      const label = createCreatureLabel(this, cx, cy, this.tileSize);
      label.setDepth(45);
      return label;
    });

    // Also exclude creature positions so chests don't overlap
    const chestExcludedPositions = [
      ...excludedPositions,
      ...this.creatures.map(c => c.position),
    ];
    this.chests = generateChestsForRoom(this.tiles, chestExcludedPositions);
    this.chestTexts = this.chests.map((c) => {
      const px = this.offsetX + c.position.x * this.tileSize;
      const py = this.offsetY + c.position.y * this.tileSize;
      return c.opened ? null : drawChestText(this, px, py, this.tileSize);
    });

    this.chaseTarget = null;
    this.chaseTickAccum = 0;
  }

  private destroyChestTexts(): void {
    for (const t of this.chestTexts) {
      if (t) t.destroy();
    }
    this.chestTexts = [];
  }

  private destroyCreatureLabels(): void {
    for (const label of this.creatureLabels) {
      label.destroy();
    }
    this.creatureLabels = [];
  }

  private drawCurrentRoom(): void {
    this.roomGraphics.clear();
    this.vignetteGraphics.clear();
    this.playerGraphics.clear();
    this.creatureGraphics.clear();
    this.chestGraphics.clear();

    drawRoom(this.roomGraphics, this.tiles, this.tileSize, this.offsetX, this.offsetY, this.elapsed);

    this.drawPlayer();
    this.drawChests();
    this.drawCreatures();
    this.drawVignette();
  }

  private drawChests(): void {
    for (let i = 0; i < this.chests.length; i++) {
      const c = this.chests[i];
      const px = this.offsetX + c.position.x * this.tileSize;
      const py = this.offsetY + c.position.y * this.tileSize;
      const isHighlighted = this.adjacentChest?.id === c.id;

      if (c.opened) {
        drawOpenChest(this.chestGraphics, px, py, this.tileSize);
      } else {
        drawChest(this.chestGraphics, px, py, this.tileSize, isHighlighted);
      }

      if (this.chestTexts[i]) {
        if (c.opened) {
          this.chestTexts[i]!.setVisible(false);
        } else {
          const cx = Math.round(px + this.tileSize / 2);
          const cy = Math.round(py + this.tileSize / 2 + this.tileSize * 0.01);
          this.chestTexts[i]!.setPosition(cx, cy);
        }
      }
    }
  }

  private drawCreatures(): void {
    for (let i = 0; i < this.creatures.length; i++) {
      const c = this.creatures[i];
      const px = this.offsetX + c.position.x * this.tileSize;
      const py = this.offsetY + c.position.y * this.tileSize;
      drawCreatureTriangle(this.creatureGraphics, px, py, this.tileSize, c.difficulty);

      const labelCx = Math.round(px + this.tileSize / 2);
      const labelCy = Math.round(py + this.tileSize / 2 + this.tileSize * 0.06);
      if (this.creatureLabels[i]) {
        this.creatureLabels[i].setPosition(labelCx, labelCy);
      }
    }
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
    this.lastMoveFromX = this.playerX;
    this.lastMoveFromY = this.playerY;
    this.moveToX = tx;
    this.moveToY = ty;
    this.moveStartTime = this.time.now;
    this.playerX = tx;
    this.playerY = ty;
    this.gameState.setPlayerPosition(tx, ty);
    if (!this.checkChestInteraction(tx, ty)) {
      this.checkEncounter(tx, ty);
    }
  }

  private checkChestInteraction(tx: number, ty: number): boolean {
    for (const c of this.chests) {
      if (c.opened) continue;
      if (c.position.x === tx && c.position.y === ty) {
        this.openChest(c);
        return true;
      }
    }
    return false;
  }

  private checkEncounter(tx: number, ty: number): void {
    for (const c of this.creatures) {
      if (c.stunned) continue;
      if (c.position.x === tx && c.position.y === ty) {
        this.gameState.emitEncounterStarted(c.id);
        return;
      }
    }
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

        void this.spawnCreatures();

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

  private updateCreatures(): void {
    const now = this.time.now;

    for (const c of this.creatures) {
      if (c.stunned) {
        if (now >= c.stunTimer) {
          c.stunned = false;
          c.stunTimer = 0;
          c.aggro = false;
        }
        continue;
      }

      if (!c.aggro) {
        const dist = Math.max(
          Math.abs(c.position.x - this.playerX),
          Math.abs(c.position.y - this.playerY),
        );
        if (dist <= c.aggroRange) {
          c.aggro = true;
        }
      }
    }

    let nearest: CreatureState | null = null;
    let nearestDist = Infinity;
    for (const c of this.creatures) {
      if (!c.aggro || c.stunned) continue;
      const dist = Math.abs(c.position.x - this.playerX) + Math.abs(c.position.y - this.playerY);
      if (dist < nearestDist) {
        nearestDist = dist;
        nearest = c;
      }
    }
    this.chaseTarget = nearest;

    if (this.chaseTarget) {
      const chaseSpeed = GAME_CONFIG.creatureChaseSpeed;
      this.chaseTickAccum += chaseSpeed;
      while (this.chaseTickAccum >= 1) {
        this.chaseTickAccum -= 1;
        this.moveCreatureToward(this.chaseTarget, this.playerX, this.playerY);
      }
    } else {
      this.chaseTickAccum = 0;
    }
  }

  private moveCreatureToward(creature: CreatureState, targetX: number, targetY: number): void {
    if (creature.stunned) return;

    const dx = Math.sign(targetX - creature.position.x);
    const dy = Math.sign(targetY - creature.position.y);

    if (dx !== 0) {
      const nx = creature.position.x + dx;
      if (this.isWalkable(nx, creature.position.y)) {
        creature.position.x = nx;
      }
    } else if (dy !== 0) {
      const ny = creature.position.y + dy;
      if (this.isWalkable(creature.position.x, ny)) {
        creature.position.y = ny;
      }
    }

    if (creature.position.x === this.playerX && creature.position.y === this.playerY) {
      this.gameState.emitEncounterStarted(creature.id);
    }
  }

  private isWalkable(x: number, y: number): boolean {
    if (x < 0 || x >= GAME_CONFIG.room.width || y < 0 || y >= GAME_CONFIG.room.height) return false;
    return this.tiles[y][x] !== TILE.WALL;
  }

  private lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
  }

  movePlayerBack(): void {
    this.playerX = this.lastMoveFromX;
    this.playerY = this.lastMoveFromY;
    this.renderX = this.lastMoveFromX;
    this.renderY = this.lastMoveFromY;
    this.gameState.setPlayerPosition(this.lastMoveFromX, this.lastMoveFromY);
  }

  stunCreature(creatureId: string): void {
    for (const c of this.creatures) {
      if (c.id === creatureId) {
        c.stunned = true;
        c.stunTimer = this.time.now + GAME_CONFIG.fleeStunDuration;
        c.aggro = false;
        break;
      }
    }
  }

  getCreatures(): CreatureState[] {
    return this.creatures;
  }

  findCreatureById(id: string): CreatureState | undefined {
    return this.creatures.find(c => c.id === id)
  }

  private openChest(chest: ChestState): void {
    if (chest.opened) return;
    chest.opened = true;
    this.gameState.generateChestLootItems();
  }

  private interactWithAdjacentChest(): void {
    for (const c of this.chests) {
      if (c.opened) continue;
      for (const [dx, dy] of DIRS) {
        const nx = this.playerX + dx;
        const ny = this.playerY + dy;
        if (c.position.x === nx && c.position.y === ny) {
          this.openChest(c);
          return;
        }
      }
    }
  }

  removeCreature(creatureId: string): void {
    const idx = this.creatures.findIndex(c => c.id === creatureId);
    if (idx !== -1) {
      if (this.creatureLabels[idx]) this.creatureLabels[idx].destroy();
      this.creatures.splice(idx, 1);
      this.creatureLabels.splice(idx, 1);
    }
    if (this.chaseTarget?.id === creatureId) {
      this.chaseTarget = null;
    }
  }

  update(_time: number, delta: number): void {
    this.elapsed += delta;
    if (this.gameState.combatActive || this.gameState.encounterActive) {
      this.drawCurrentRoom();
      return;
    }
    this.updateAdjacentChest();
    this.processInput();
    this.updateMovement();
    this.updateCreatures();
    this.drawCurrentRoom();
  }

  private updateAdjacentChest(): void {
    this.adjacentChest = null;
    for (const c of this.chests) {
      if (c.opened) continue;
      for (const [dx, dy] of DIRS) {
        const nx = this.playerX + dx;
        const ny = this.playerY + dy;
        if (c.position.x === nx && c.position.y === ny) {
          this.adjacentChest = c;
          return;
        }
      }
    }
  }
}
