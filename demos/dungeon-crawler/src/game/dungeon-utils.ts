import { TILE, type TileType } from './room-tiles';

export function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

export function pickRandom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

export function chebyshev(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return Math.max(Math.abs(a.x - b.x), Math.abs(a.y - b.y));
}

export function getAvailableFloorTiles(
  tiles: TileType[][],
  exclusionCenters: { x: number; y: number }[],
  exclusionRadius: number,
): { x: number; y: number }[] {
  const rw = tiles[0]?.length ?? 0;
  const rh = tiles.length;
  const floorTiles: { x: number; y: number }[] = [];
  for (let y = 0; y < rh; y++) {
    for (let x = 0; x < rw; x++) {
      if (tiles[y][x] === TILE.FLOOR) {
        const isExcluded = exclusionCenters.some(
          center => chebyshev({ x, y }, center) < exclusionRadius,
        );
        if (!isExcluded) {
          floorTiles.push({ x, y });
        }
      }
    }
  }
  return floorTiles;
}
