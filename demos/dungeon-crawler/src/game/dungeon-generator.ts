import { GAME_CONFIG } from '../config/game-config';

const DIRS: [number, number][] = [[0, -1], [0, 1], [-1, 0], [1, 0]];

export interface RoomData {
  x: number;
  y: number;
  id: number;
}

export interface Edge {
  from: number;
  to: number;
}

export class Dungeon {
  rooms: RoomData[] = [];
  grid: number[][] = [];
  treeEdges: Edge[] = [];
  extraEdges: Edge[] = [];
  adj: Set<number>[] = [];
  connectionSet: Set<string> = new Set();
  bossRoom = -1;
  entranceRoom = 0;

  constructor() {
    this.generate();
  }

  generate(): void {
    const gs = GAME_CONFIG.dungeon.gridSize;
    this.grid = Array.from({ length: gs }, () => Array(gs).fill(-1));

    const startX = rand(1, Math.max(1, gs - 2));
    const startY = rand(1, Math.max(1, gs - 2));

    this.rooms = [{ x: startX, y: startY, id: 0 }];
    this.grid[startY][startX] = 0;

    const targetCount = Math.floor(gs * gs * GAME_CONFIG.dungeon.targetRoomRatio);

    const frontierSet = new Set<string>();
    const frontier: { x: number; y: number; fromX: number; fromY: number }[] = [];

    const addFrontier = (x: number, y: number, fx: number, fy: number): void => {
      if (x < 0 || x >= gs || y < 0 || y >= gs) return;
      if (this.grid[y][x] !== -1) return;
      const key = `${x},${y}`;
      if (frontierSet.has(key)) return;
      frontierSet.add(key);
      frontier.push({ x, y, fromX: fx, fromY: fy });
    };

    for (const [dx, dy] of DIRS) {
      addFrontier(startX + dx, startY + dy, startX, startY);
    }

    this.treeEdges = [];

    while (this.rooms.length < targetCount && frontier.length > 0) {
      const idx = Math.floor(Math.random() * frontier.length);
      const cell = frontier[idx];
      frontier[idx] = frontier[frontier.length - 1];
      frontier.pop();
      frontierSet.delete(`${cell.x},${cell.y}`);

      if (this.grid[cell.y][cell.x] !== -1) continue;

      const newId = this.rooms.length;
      this.rooms.push({ x: cell.x, y: cell.y, id: newId });
      this.grid[cell.y][cell.x] = newId;
      const fromId = this.grid[cell.fromY][cell.fromX];
      this.treeEdges.push({ from: fromId, to: newId });

      for (const [dx, dy] of DIRS) {
        addFrontier(cell.x + dx, cell.y + dy, cell.x, cell.y);
      }
    }

    const n = this.rooms.length;
    this.adj = Array.from({ length: n }, () => new Set<number>());

    const roomPosMap = new Map<string, number>();
    for (let i = 0; i < n; i++) {
      roomPosMap.set(`${this.rooms[i].x},${this.rooms[i].y}`, i);
    }
    for (let i = 0; i < n; i++) {
      const r = this.rooms[i];
      for (const [dx, dy] of DIRS) {
        const nb = roomPosMap.get(`${r.x + dx},${r.y + dy}`);
        if (nb !== undefined) {
          this.adj[i].add(nb);
        }
      }
    }

    const treeSet = new Set<string>();
    for (const e of this.treeEdges) {
      treeSet.add(`${e.from},${e.to}`);
      treeSet.add(`${e.to},${e.from}`);
    }

    const nonTreeEdges: Edge[] = [];
    for (let i = 0; i < n; i++) {
      for (const j of this.adj[i]) {
        if (j > i && !treeSet.has(`${i},${j}`)) {
          nonTreeEdges.push({ from: i, to: j });
        }
      }
    }

    shuffle(nonTreeEdges);
    const extraCount = Math.floor(nonTreeEdges.length * GAME_CONFIG.dungeon.extraEdgeRatio);
    this.extraEdges = nonTreeEdges.slice(0, extraCount);

    this.connectionSet = new Set<string>();
    for (const e of this.treeEdges) {
      this.connectionSet.add(`${e.from},${e.to}`);
      this.connectionSet.add(`${e.to},${e.from}`);
    }
    for (const e of this.extraEdges) {
      this.connectionSet.add(`${e.from},${e.to}`);
      this.connectionSet.add(`${e.to},${e.from}`);
    }

    const dist = Array(n).fill(-1);
    const q = [0];
    dist[0] = 0;
    for (let qi = 0; qi < q.length; qi++) {
      const cur = q[qi];
      for (const nb of this.adj[cur]) {
        if (dist[nb] === -1 && this.connectionSet.has(`${cur},${nb}`)) {
          dist[nb] = dist[cur] + 1;
          q.push(nb);
        }
      }
    }
    let maxDist = -1;
    for (let i = 0; i < n; i++) {
      if (dist[i] > maxDist) {
        maxDist = dist[i];
        this.bossRoom = i;
      }
    }
    this.entranceRoom = 0;
  }

  getNeighbor(roomId: number, dir: number): number {
    const r = this.rooms[roomId];
    const [dx, dy] = DIRS[dir];
    const nx = r.x + dx;
    const ny = r.y + dy;
    if (nx >= 0 && nx < GAME_CONFIG.dungeon.gridSize && ny >= 0 && ny < GAME_CONFIG.dungeon.gridSize) {
      const nb = this.grid[ny][nx];
      if (nb !== -1) return nb;
    }
    return -1;
  }

  hasDoor(roomId: number, dir: number): boolean {
    const nb = this.getNeighbor(roomId, dir);
    return nb !== -1 && this.connectionSet.has(`${roomId},${nb}`);
  }
}

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function shuffle<T>(arr: T[]): void {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}
