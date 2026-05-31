import type { CreatureState } from '../types'
import './EnemyCard.css'

const DIFFICULTY_COLORS: Record<string, string> = {
  normal: '#88b898',
  champion: '#ffa500',
  elite: '#ff4444',
  boss: '#aa44ff',
}

interface EnemyCardProps {
  creature: CreatureState
}

function EnemyCard({ creature }: EnemyCardProps) {
  const hpPercent = creature.hp.max > 0 ? (creature.hp.current / creature.hp.max) * 100 : 0
  const badgeColor = DIFFICULTY_COLORS[creature.difficulty] ?? '#88b898'

  return (
    <div id="enemy-card">
      <div className="ph">
        <div className="pa"><i className="fa-solid fa-dragon" /></div>
        <div>
          <div className="pn">{creature.name}</div>
          <div className="pl" style={{ color: badgeColor }}>LV {creature.level} · {creature.difficulty.toUpperCase()}</div>
        </div>
      </div>
      <div className="pb">
        <div className="br">
          <span className="lbl"><i className="fa-solid fa-heart" /></span>
          <div className="bb"><div className="bf hp" style={{ width: `${hpPercent}%` }} /></div>
          <span className="bt">{creature.hp.current}/{creature.hp.max}</span>
        </div>
      </div>
      <div className="ps">
        <span><i className="fa-solid fa-crosshairs" /> ATK {creature.attack}</span>
        <span><i className="fa-solid fa-shield-halved" /> DEF {creature.defense}</span>
      </div>
    </div>
  )
}

export default EnemyCard
