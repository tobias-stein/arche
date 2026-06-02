import { useState, useEffect, useRef, useCallback } from 'react';
import { getGameState } from '../GameState';
import { GAME_CONFIG } from '../config';
import type { Dungeon } from '../game/dungeon-generator';
import './MiniMap.css';

const SM_HALF = 3;
const SM_CELL = 18;
const SM_GAP = 2;
const SM_PAD = 4;

const LG_GAP = 3;
const LG_PAD = 8;
const ZOOM_MIN = 0.25;
const ZOOM_MAX = 5;

function MiniMap() {
  const [dungeon, setDungeon] = useState<Dungeon | null>(null);
  const [currentRoom, setCurrentRoom] = useState(0);
  const [visitedRooms, setVisitedRooms] = useState<Set<number>>(new Set());
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [zoom, setZoom] = useState(1);

  const smallCanvasRef = useRef<HTMLCanvasElement>(null);
  const largeCanvasRef = useRef<HTMLCanvasElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const panRef = useRef({ panning: false, startX: 0, startY: 0, scrollLeft: 0, scrollTop: 0 });
  const shouldCenterOnOpen = useRef(false);
  const prevZoomRef = useRef(zoom);

  useEffect(() => {
    const gs = getGameState();
    if (gs.dungeon) {
      setDungeon(gs.dungeon);
      setCurrentRoom(gs.currentRoomId);
      setVisitedRooms(new Set(gs.visitedRooms));
    }

    function onDungeonReady(d: Dungeon) {
      setDungeon(d);
      setCurrentRoom(0);
      setVisitedRooms(new Set([0]));
    }

    function onRoomChanged(roomId: number) {
      setCurrentRoom(roomId);
      setVisitedRooms(new Set(getGameState().visitedRooms));
    }

    gs.on('dungeon:ready', onDungeonReady);
    gs.on('room:changed', onRoomChanged);
    return () => {
      gs.off('dungeon:ready', onDungeonReady);
      gs.off('room:changed', onRoomChanged);
    };
  }, []);

  useEffect(() => {
    if (!dungeon || !smallCanvasRef.current) return;
    drawSmallMinimap();
  });

  useEffect(() => {
    if (!dungeon || !largeCanvasRef.current || !overlayOpen) return;
    drawLargeMinimap();
  });

  useEffect(() => {
    if (overlayOpen && dungeon) {
      setZoom(ZOOM_MAX);
      shouldCenterOnOpen.current = true;
      prevZoomRef.current = ZOOM_MAX;
    }
  }, [overlayOpen]);

  useEffect(() => {
    if (!overlayOpen || !dungeon) return;
    if (shouldCenterOnOpen.current && zoom === ZOOM_MAX) {
      scrollToPlayer();
      shouldCenterOnOpen.current = false;
    }
  });

  useEffect(() => {
    if (!overlayOpen || !dungeon || shouldCenterOnOpen.current) return;
    const prevZoom = prevZoomRef.current;
    if (prevZoom === zoom) return;
    const el = scrollRef.current;
    if (!el) return;
    const prevCell = Math.max(2, Math.round(10 * prevZoom));
    const newCell = Math.max(2, Math.round(10 * zoom));
    if (prevCell === newCell) return;
    const gridSize = GAME_CONFIG.dungeon.gridSize;
    const centerX = el.scrollLeft + el.clientWidth / 2;
    const centerY = el.scrollTop + el.clientHeight / 2;
    const gx = (centerX - LG_PAD) / (prevCell + LG_GAP);
    const gy = (centerY - LG_PAD) / (prevCell + LG_GAP);
    el.scrollLeft = Math.max(0, LG_PAD + gx * (newCell + LG_GAP) - el.clientWidth / 2);
    el.scrollTop = Math.max(0, LG_PAD + gy * (newCell + LG_GAP) - el.clientHeight / 2);
    prevZoomRef.current = zoom;
  });

  const scrollToPlayer = useCallback(() => {
    const el = scrollRef.current;
    const dun = dungeon;
    if (!el || !dun || el.clientWidth === 0) return;
    const gs = GAME_CONFIG.dungeon.gridSize;
    const cw = largeCanvasRef.current?.width ?? 0;
    const cell = cw > 0 ? (cw - 16 - (gs - 1) * LG_GAP) / gs : 10;
    const room = dun.rooms[currentRoom];
    const px = LG_PAD + room.x * (cell + LG_GAP) + cell / 2;
    const py = LG_PAD + room.y * (cell + LG_GAP) + cell / 2;
    el.scrollLeft = Math.max(0, px - el.clientWidth / 2);
    el.scrollTop = Math.max(0, py - el.clientHeight / 2);
  }, [dungeon, currentRoom]);

  function drawSmallMinimap() {
    const dun = dungeon;
    const vr = visitedRooms;
    if (!dun || !smallCanvasRef.current) return;
    const canvas = smallCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const pr = dun.rooms[currentRoom];
    if (!pr) return;
    const gridSize = GAME_CONFIG.dungeon.gridSize;
    const dim = SM_HALF * 2 + 1;
    const w = dim * SM_CELL + (dim - 1) * SM_GAP + SM_PAD * 2;
    const h = dim * SM_CELL + (dim - 1) * SM_GAP + SM_PAD * 2;

    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let dy = -SM_HALF; dy <= SM_HALF; dy++) {
      for (let dx = -SM_HALF; dx <= SM_HALF; dx++) {
        const gx = pr.x + dx;
        const gy = pr.y + dy;
        if (gx < 0 || gx >= gridSize || gy < 0 || gy >= gridSize) continue;
        const rid = dun.grid[gy][gx];
        if (rid === -1) continue;
        if (!vr.has(rid) && rid !== dun.bossRoom) continue;

        const rx = SM_PAD + (dx + SM_HALF) * (SM_CELL + SM_GAP);
        const ry = SM_PAD + (dy + SM_HALF) * (SM_CELL + SM_GAP);

        if (rid === currentRoom) {
          ctx.fillStyle = '#ffd700';
        } else if (rid === dun.bossRoom) {
          ctx.fillStyle = vr.has(rid) ? '#e53935' : '#2a1515';
        } else {
          ctx.fillStyle = '#5a5a6a';
        }
        ctx.fillRect(rx, ry, SM_CELL, SM_CELL);

        if (rid === currentRoom) {
          ctx.fillStyle = '#4fc3f7';
          const s = Math.floor(SM_CELL * 0.4);
          ctx.fillRect(rx + (SM_CELL - s) / 2, ry + (SM_CELL - s) / 2, s, s);
        }

        ctx.strokeStyle = rid === currentRoom ? '#fff' : '#333';
        ctx.lineWidth = rid === currentRoom ? 2 : 0.5;
        ctx.strokeRect(rx, ry, SM_CELL, SM_CELL);

        if (rid === dun.bossRoom && !vr.has(rid) && SM_CELL >= 6) {
          ctx.fillStyle = '#c33';
          ctx.font = `bold ${Math.min(SM_CELL - 2, 10)}px sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText('?', rx + SM_CELL / 2, ry + SM_CELL / 2 + 1);
        }

        if (rid === dun.entranceRoom && vr.has(rid)) {
          ctx.fillStyle = '#66bb6a';
          ctx.fillRect(rx + 2, ry + 2, 4, 4);
        }
      }
    }
  }

  function drawLargeMinimap() {
    const dun = dungeon;
    const vr = visitedRooms;
    if (!dun || !largeCanvasRef.current) return;
    const canvas = largeCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const gridSize = GAME_CONFIG.dungeon.gridSize;
    const cell = Math.max(2, Math.round(10 * zoom));

    const cw = gridSize * cell + (gridSize - 1) * LG_GAP + LG_PAD * 2;
    const ch = gridSize * cell + (gridSize - 1) * LG_GAP + LG_PAD * 2;
    canvas.width = cw;
    canvas.height = ch;

    ctx.lineWidth = Math.max(1, cell * 0.08);
    const allEdges = [...dun.treeEdges, ...dun.extraEdges];
    for (const e of allEdges) {
      if (!vr.has(e.from) || !vr.has(e.to)) continue;
      const a = dun.rooms[e.from];
      const b = dun.rooms[e.to];
      const highlight = e.from === currentRoom || e.to === currentRoom;
      ctx.strokeStyle = highlight ? '#ffd700' : cell > 6 ? '#666' : '#555';
      const ax = LG_PAD + a.x * (cell + LG_GAP) + cell / 2;
      const ay = LG_PAD + a.y * (cell + LG_GAP) + cell / 2;
      const bx = LG_PAD + b.x * (cell + LG_GAP) + cell / 2;
      const by = LG_PAD + b.y * (cell + LG_GAP) + cell / 2;
      ctx.beginPath();
      ctx.moveTo(ax, ay);
      ctx.lineTo(bx, by);
      ctx.stroke();
    }

    for (let i = 0; i < dun.rooms.length; i++) {
      const isUnvisited = !vr.has(i);
      if (isUnvisited && i !== dun.bossRoom) continue;
      const r = dun.rooms[i];
      const rx = LG_PAD + r.x * (cell + LG_GAP);
      const ry = LG_PAD + r.y * (cell + LG_GAP);

      if (isUnvisited) {
        ctx.fillStyle = '#000';
        ctx.fillRect(rx, ry, cell, cell);
        ctx.strokeStyle = '#900';
        ctx.lineWidth = Math.max(1, cell * 0.12);
        ctx.strokeRect(rx, ry, cell, cell);
        if (cell >= 8) {
          ctx.fillStyle = '#c33';
          ctx.font = `bold ${Math.min(cell - 2, 14)}px sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText('?', rx + cell / 2, ry + cell / 2 + 1);
        }
      } else if (i === currentRoom) {
        ctx.fillStyle = '#ffd700';
        ctx.fillRect(rx, ry, cell, cell);
        ctx.strokeStyle = cell > 6 ? '#fff' : '#ffd700';
        ctx.lineWidth = Math.max(1, cell * 0.1);
        ctx.strokeRect(rx, ry, cell, cell);
      } else {
        ctx.fillStyle = i === dun.bossRoom ? '#e53935' : cell > 6 ? '#5a5a6a' : '#444';
        ctx.fillRect(rx, ry, cell, cell);
        ctx.strokeStyle = cell > 6 ? '#333' : '#222';
        ctx.lineWidth = 1;
        ctx.strokeRect(rx, ry, cell, cell);
      }

      if (i === dun.bossRoom && !isUnvisited && cell >= 8) {
        ctx.fillStyle = '#fff';
        ctx.font = `${Math.min(cell - 2, 16)}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('\u2620', rx + cell / 2, ry + cell / 2 + 1);
      }

      if (i === dun.entranceRoom && vr.has(i) && cell >= 6) {
        ctx.fillStyle = '#66bb6a';
        ctx.fillRect(rx + 2, ry + 2, 4, 4);
      }
    }

    if (cell >= 8) {
      ctx.fillStyle = '#888';
      ctx.font = '9px monospace';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'bottom';
      ctx.fillText('\u25a0 Current   \u2620 Boss   \u2756 Entrance', LG_PAD, canvas.height - 3);
    }
  }

  function handleZoomChange(level: number) {
    setZoom(Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, level)));
  }

  function handleSliderInput(e: React.ChangeEvent<HTMLInputElement>) {
    handleZoomChange(parseInt(e.target.value, 10) / 100);
  }

  function handleMouseDown(e: React.MouseEvent) {
    if (e.button !== 0) return;
    const el = scrollRef.current;
    if (!el) return;
    panRef.current.panning = true;
    panRef.current.startX = e.clientX;
    panRef.current.startY = e.clientY;
    panRef.current.scrollLeft = el.scrollLeft;
    panRef.current.scrollTop = el.scrollTop;
    el.classList.add('panning');
  }

  useEffect(() => {
    function handleMouseMove(e: MouseEvent) {
      const p = panRef.current;
      if (!p.panning) return;
      const el = scrollRef.current;
      if (!el) return;
      el.scrollLeft = p.scrollLeft - (e.clientX - p.startX);
      el.scrollTop = p.scrollTop - (e.clientY - p.startY);
    }
    function handleMouseUp() {
      const p = panRef.current;
      if (p.panning) {
        p.panning = false;
        const el = scrollRef.current;
        if (el) el.classList.remove('panning');
      }
    }
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  function openOverlay() {
    const gs = getGameState();
    if (gs.encounterActive || gs.combatActive || gs.gameOver || gs.victory) return;
    setOverlayOpen(true);
  }

  function closeOverlay() {
    setOverlayOpen(false);
  }

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'm' || e.key === 'M') {
        const gs = getGameState();
        if (gs.encounterActive || gs.combatActive || gs.gameOver || gs.victory) return;
        e.preventDefault();
        setOverlayOpen(prev => !prev);
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const zoomLabel = zoom % 1 === 0 ? zoom.toFixed(0) : zoom.toFixed(2).replace(/\.?0+$/, '');

  return (
    <>
      <div
        id="minimap-small"
        data-testid="minimap-small"
        onClick={openOverlay}
        className={overlayOpen ? 'hidden' : ''}
      >
        <canvas ref={smallCanvasRef} data-testid="mini-canvas-small" />
      </div>

      {overlayOpen && (
        <div id="minimap-wrap" data-testid="minimap-overlay">
          <div
            className="map-scroll"
            ref={scrollRef}
            onMouseDown={handleMouseDown}
          >
            <canvas ref={largeCanvasRef} data-testid="mini-canvas-large" />
          </div>
          <div className="minimap-zoom">
            <input
              type="range"
              min={25}
              max={500}
              step={25}
              value={Math.round(zoom * 100)}
              onChange={handleSliderInput}
              aria-label="Zoom"
            />
            <span className="level">{zoomLabel}\u00d7</span>
            <button className="recenter" onClick={scrollToPlayer} aria-label="Center on player">{'\u2316'}</button>
          </div>
          <div className="label">Drag to pan \u00b7 <kbd>M</kbd> close</div>
          <button id="minimap-close" aria-label="Close minimap" onClick={closeOverlay}>
            &times;
          </button>
        </div>
      )}
    </>
  );
}

export default MiniMap;
