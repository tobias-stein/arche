type Difficulty = 'normal' | 'champion' | 'elite' | 'boss'
type Rarity = 'common' | 'uncommon' | 'rare' | 'legendary'
type EquipSlot = 'weapon' | 'helmet' | 'chest' | 'legs' | 'boots' | 'gloves' | 'belt' | 'ring' | 'amulet' | 'shield'

interface PlayerState {
  name: string
  level: number
  xp: { current: number; next: number }
  hp: { current: number; max: number }
  mp: { current: number; max: number }
  attack: number
  defense: number
  position: { x: number; y: number }
}

interface CreatureState {
  id: string
  name: string
  archeType: string
  subtype: string
  difficulty: Difficulty
  level: number
  hp: { current: number; max: number }
  attack: number
  defense: number
  xpReward: number
  position: { x: number; y: number }
  aggroRange: number
  aggro: boolean
  stunned: boolean
  stunTimer: number
}

interface ChestState {
  id: string
  position: { x: number; y: number }
  opened: boolean
}

interface ItemState {
  id: string
  name: string
  archeType: string
  subtype?: string
  rarity: Rarity
  level: number
  equipSlot?: EquipSlot
  stats: Record<string, number>
  affixes: string[]
}

type LogEventType =
  | 'player_damage_dealt'
  | 'player_damage_taken'
  | 'enemy_slain'
  | 'spell_cast'
  | 'potion_consumed'
  | 'item_picked_up'
  | 'item_dropped'
  | 'room_entered'
  | 'encounter_started'
  | 'level_up'
  | 'flee_attempt'

interface LogEvent {
  type: LogEventType
  message: string
  icon: string
}

export type { Difficulty, Rarity, EquipSlot, PlayerState, CreatureState, ChestState, ItemState, LogEventType, LogEvent }
