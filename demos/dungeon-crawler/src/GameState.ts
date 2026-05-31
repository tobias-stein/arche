import { GAME_CONFIG } from './config'
import type { PlayerState, LogEvent } from './types'

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
