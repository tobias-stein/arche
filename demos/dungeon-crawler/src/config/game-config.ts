export const GAME_CONFIG = {
  player: {
    baseHp: 100,
    baseMp: 30,
    baseAttack: 10,
    baseDefense: 5,
    startingLevel: 1,
    maxLevel: 100,
  },

  dungeon: {
    gridSize: 64,
    targetRoomRatio: 0.55,
    extraEdgeRatio: 0.20,
    transitionMs: 350,
  },

  room: {
    width: 15,
    height: 11,
  },

  movement: {
    walkSpeed: 120,
    holdMoveInterval: 110,
  },

  generationWindow: {
    creatureLevelVariance: 2,
    itemLevelVariance: 2,
  },

  creatureSpawn: {
    minPerRoom: 1,
    maxPerRoom: 4,
    entryExclusionRadius: 4,
  },

  chestSpawn: {
    minPerRoom: 0,
    maxPerRoom: 3,
    entryExclusionRadius: 4,
  },

  aggroRanges: {
    normal: 1,
    champion: 2,
    elite: 3,
    boss: 3,
  },

  creatureChaseSpeed: 0.5,

  fleeStunDuration: 1500,
} as const;
