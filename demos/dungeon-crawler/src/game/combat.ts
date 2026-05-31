import { GAME_CONFIG } from '../config'
import type { CreatureState, PlayerState } from '../types'

export function calculatePlayerDamage(player: PlayerState, creature: CreatureState): number {
  const roll = Math.floor(Math.random() * (GAME_CONFIG.combat.playerDamageRollMax + 1))
  return Math.max(GAME_CONFIG.combat.minDamage, player.attack - creature.defense + roll)
}

export function calculateCreatureDamage(player: PlayerState, creature: CreatureState): number {
  const roll = Math.floor(Math.random() * (GAME_CONFIG.combat.creatureDamageRollMax + 1))
  return Math.max(
    GAME_CONFIG.combat.minDamage,
    creature.attack - player.defense + roll - GAME_CONFIG.combat.creatureAttackPenalty,
  )
}
