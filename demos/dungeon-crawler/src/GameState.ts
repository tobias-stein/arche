import { GAME_CONFIG } from './config'
import type { PlayerState, CreatureState, LogEvent } from './types'
import type { Dungeon } from './game/dungeon-generator'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type EventCallback = (...args: any[]) => void

class EventEmitter {
  private listeners = new Map<string, EventCallback[]>()

  on(event: string, callback: EventCallback): void {
    if (!this.listeners.has(event)) this.listeners.set(event, [])
    this.listeners.get(event)!.push(callback)
  }

  off(event: string, callback: EventCallback): void {
    const cbs = this.listeners.get(event)
    if (!cbs) return
    this.listeners.set(event, cbs.filter(cb => cb !== callback))
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  emit(event: string, ...args: any[]): void {
    this.listeners.get(event)?.forEach(cb => cb(...args))
  }
}

class GameState extends EventEmitter {
  player: PlayerState
  gameStarted = false
  currentRoomId = 0
  visitedRooms: Set<number> = new Set()
  dungeon: Dungeon | null = null

  combatActive = false
  combatCreature: CreatureState | null = null
  combatTurn: 'player' | 'enemy' | null = null
  combatVictory = false
  combatDefeat = false

  constructor() {
    super()
    const cfg = GAME_CONFIG.player
    const nextXp = GAME_CONFIG.xpThresholds[cfg.startingLevel - 1] ?? 10
    this.player = {
      name: 'Hero',
      level: cfg.startingLevel,
      xp: { current: 0, next: nextXp },
      hp: { current: cfg.baseHp, max: cfg.baseHp },
      mp: { current: cfg.baseMp, max: cfg.baseMp },
      attack: cfg.baseAttack,
      defense: cfg.baseDefense,
      position: { x: 0, y: 0 },
    }
  }

  startGame(): void {
    this.gameStarted = true
    this.emit('game:started')
  }

  setDungeonData(dungeon: Dungeon): void {
    this.dungeon = dungeon
    this.emit('dungeon:ready', dungeon)
  }

  setPlayerPosition(x: number, y: number): void {
    this.player.position.x = x
    this.player.position.y = y
    this.emit('player:moved', this.player)
  }

  setCurrentRoom(roomId: number): void {
    this.currentRoomId = roomId
    this.visitedRooms.add(roomId)
    this.emit('room:changed', roomId)
  }

  setPlayerHp(value: number): void {
    this.player.hp.current = Math.max(0, Math.min(value, this.player.hp.max))
    this.emit('player:stats-changed', this.player)
  }

  setPlayerMp(value: number): void {
    this.player.mp.current = Math.max(0, Math.min(value, this.player.mp.max))
    this.emit('player:stats-changed', this.player)
  }

  setPlayerAttack(value: number): void {
    this.player.attack = value
    this.emit('player:stats-changed', this.player)
  }

  setPlayerDefense(value: number): void {
    this.player.defense = value
    this.emit('player:stats-changed', this.player)
  }

  addXp(amount: number): void {
    this.player.xp.current += amount
    while (this.player.xp.current >= this.player.xp.next && this.player.level < GAME_CONFIG.player.maxLevel) {
      this.player.xp.current -= this.player.xp.next
      this.player.level++
      this.player.xp.next = GAME_CONFIG.xpThresholds[this.player.level - 1] ?? this.player.xp.next
      this.emit('xp:level-up', this.player.level)
    }
    this.emit('player:stats-changed', this.player)
  }

  addLogEntry(entry: LogEvent): void {
    this.emit('log:entry', entry)
  }

  emitInventoryRequested(): void {
    this.emit('inventory:requested')
  }

  emitEncounterStarted(creatureId: string): void {
    this.emit('encounter:started', creatureId)
  }

  startCombat(creature: CreatureState): void {
    this.combatActive = true
    this.combatCreature = creature
    this.combatTurn = 'player'
    this.combatVictory = false
    this.combatDefeat = false
    this.emit('combat:started', creature)
  }

  processPlayerAttack(damage: number): void {
    if (!this.combatCreature) return
    this.combatCreature.hp.current = Math.max(0, this.combatCreature.hp.current - damage)
    this.emit('combat:creature-damaged', damage, this.combatCreature)

    if (this.combatCreature.hp.current <= 0) {
      this.resolveVictory()
    } else {
      this.combatTurn = 'enemy'
      this.emit('combat:turn-changed', 'enemy')
    }
  }

  processEnemyTurn(damage: number): void {
    this.setPlayerHp(this.player.hp.current - damage)
    this.emit('combat:player-damaged', damage, this.player)

    if (this.player.hp.current <= 0) {
      this.resolveDefeat()
    } else {
      this.combatTurn = 'player'
      this.emit('combat:turn-changed', 'player')
    }
  }

  resolveVictory(): void {
    this.combatVictory = true
    this.combatTurn = null
    const xpReward = this.combatCreature?.xpReward ?? 0
    this.addXp(xpReward)

    const hpRestore = Math.round(this.player.hp.max * GAME_CONFIG.recovery.hpPercent)
    const mpRestore = Math.round(this.player.mp.max * GAME_CONFIG.recovery.mpPercent)
    this.setPlayerHp(this.player.hp.current + hpRestore)
    this.setPlayerMp(this.player.mp.current + mpRestore)

    this.addLogEntry({
      type: 'enemy_slain',
      message: `${this.combatCreature?.name ?? 'Enemy'} slain! +${xpReward} XP`,
      icon: 'fa-solid fa-crosshairs',
    })

    const creatureId = this.combatCreature?.id ?? ''
    this.emit('combat:victory', creatureId, xpReward)
  }

  resolveDefeat(): void {
    this.combatDefeat = true
    this.combatTurn = null
    this.emit('combat:defeat')
  }

  fleeCombat(): void {
    this.addLogEntry({
      type: 'flee_attempt',
      message: 'Fled from combat!',
      icon: 'fa-solid fa-person-running',
    })
    const creatureId = this.combatCreature?.id ?? ''
    this.emit('combat:fled', creatureId)
    this.endCombat()
  }

  endCombat(): void {
    this.combatActive = false
    this.combatCreature = null
    this.combatTurn = null
    this.combatVictory = false
    this.combatDefeat = false
    this.emit('combat:ended')
  }
}

let _instance: GameState | null = null

function getGameState(): GameState {
  if (!_instance) _instance = new GameState()
  return _instance
}

function resetGameState(): GameState {
  _instance = new GameState()
  return _instance
}

export { GameState, EventEmitter, getGameState, resetGameState }
