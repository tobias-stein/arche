import { useState, useEffect, useRef, useCallback } from 'react'
import { getGameState } from '../GameState'
import type { LogEvent, LogEventType } from '../types'
import './ActivityLog.css'

const MAX_LOG = 200
const EVENT_COLORS: Record<LogEventType, string> = {
  player_damage_dealt: '#ffffff',
  player_damage_taken: '#ff4444',
  enemy_slain: '#ff4444',
  spell_cast: '#44ddff',
  potion_consumed: '#44cc66',
  item_picked_up: '#ffff44',
  item_dropped: '#888888',
  room_entered: '#ffffff',
  encounter_started: '#ffff44',
  level_up: '#ffd700',
  flee_attempt: '#888888',
}

function formatTimestamp(startTime: number): string {
  const elapsed = Date.now() - startTime
  const totalSeconds = Math.floor(elapsed / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}

interface LogEntry {
  timestamp: string
  type: LogEventType
  message: string
  icon: string
}

let logEntryId = 0

function ActivityLog() {
  const [expanded, setExpanded] = useState(false)
  const [entries, setEntries] = useState<LogEntry[]>([])
  const [autoScroll, setAutoScroll] = useState(true)
  const bodyRef = useRef<HTMLDivElement>(null)
  const startTimeRef = useRef(Date.now())
  const programmaticRef = useRef(false)

  useEffect(() => {
    const gs = getGameState()
    startTimeRef.current = Date.now()

    function onLogEntry(event: LogEvent) {
      setEntries(prev => {
        const entry: LogEntry = {
          timestamp: formatTimestamp(startTimeRef.current),
          type: event.type,
          message: event.message,
          icon: event.icon,
        }
        const next = [entry, ...prev]
        if (next.length > MAX_LOG) next.length = MAX_LOG
        return next
      })
    }

    gs.on('log:entry', onLogEntry)
    return () => {
      gs.off('log:entry', onLogEntry)
    }
  }, [])

  useEffect(() => {
    if (autoScroll && expanded && bodyRef.current) {
      programmaticRef.current = true
      bodyRef.current.scrollTop = 0
    }
  })

  const handleScroll = useCallback(() => {
    if (programmaticRef.current) {
      programmaticRef.current = false
      return
    }
    const el = bodyRef.current
    if (!el) return
    if (el.scrollTop > 0 && autoScroll) {
      setAutoScroll(false)
    }
    if (el.scrollTop === 0 && !autoScroll) {
      setAutoScroll(true)
    }
  }, [autoScroll])

  const scrollToTop = useCallback(() => {
    setAutoScroll(true)
    programmaticRef.current = true
    if (bodyRef.current) {
      bodyRef.current.scrollTop = 0
    }
  }, [])

  const toggleExpanded = useCallback(() => {
    setExpanded(prev => !prev)
  }, [])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'l' || e.key === 'L') {
        setExpanded(prev => !prev)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    if (!expanded) {
      setAutoScroll(true)
    }
  }, [expanded])

  const displayEntries = expanded ? entries : entries.slice(0, 5)

  return (
    <div id="log" data-testid="activity-log" className={expanded ? 'expanded' : ''}>
      <div className="log-header" onClick={toggleExpanded}>
        <span><i className="fa-solid fa-scroll" /></span>
        <span className="log-title">Activity Log</span>
        <span className="log-toggle"><i className="fa-solid fa-chevron-down" /></span>
      </div>
      <div
        className="log-body"
        data-testid="log-body"
        ref={bodyRef}
        onScroll={handleScroll}
      >
        {displayEntries.map(e => (
          <div
            key={e.timestamp + e.message + (logEntryId++)}
            className="le"
            style={{ color: EVENT_COLORS[e.type] || '#b8d8c8' }}
          >
            [{e.timestamp}] <i className={`fa-solid ${e.icon}`} /> {e.message}
          </div>
        ))}
      </div>
      <div
        className={`log-scroll-btn${autoScroll || !expanded ? '' : ' visible'}`}
        onClick={scrollToTop}
      >
        <i className="fa-solid fa-arrow-up" />
      </div>
    </div>
  )
}

export default ActivityLog
