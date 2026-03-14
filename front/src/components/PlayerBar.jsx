import { useState, useEffect, useRef } from 'react';
import { CONFIG } from '../config.js';
import { Play, Pause, SkipForward, SkipBack, Square } from 'lucide-react';

const fmt = (secs) => {
  const s = Math.floor(secs);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
};

const PlayerBar = () => {
  const [state, setState] = useState(null);
  const wsRef = useRef(null);

  useEffect(() => {
    const connect = () => {
      const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(`${proto}//${window.location.host}/ws`);
      wsRef.current = ws;
      ws.onmessage = (e) => {
        try { setState(JSON.parse(e.data)); } catch {}
      };
      ws.onclose = () => setTimeout(connect, 2000);
    };
    fetch(`${CONFIG.API}/current`)
      .then(r => r.json())
      .then(data => setState(prev => ({ ...prev, ...data })))
      .catch(() => {});
    connect();
    return () => wsRef.current?.close();
  }, []);

  const cmd = async (endpoint) => {
    try { await fetch(`${CONFIG.API}/${endpoint}`, { method: 'POST' }); } catch {}
  };

  const togglePlay = () => cmd(state?.is_playing ? 'pause' : 'resume');

  const track    = state?.current ?? null;
  const playing  = state?.is_playing ?? false;
  const elapsed  = state?.elapsed_secs ?? 0;
  const duration = state?.duration_secs ?? 0;
  const progress = state?.progress != null
    ? Math.min(state.progress * 100, 100)
    : duration > 0 ? Math.min((elapsed / duration) * 100, 100) : 0;
  const connected = wsRef.current?.readyState === 1;

  return (
    <footer className="fixed bottom-0 w-full z-30"
      style={{ background: 'rgba(12,12,18,0.92)', backdropFilter: 'blur(20px)', borderTop: '1px solid rgba(100,100,130,0.12)' }}>

      {/* Barra de progreso */}
      <div className="w-full h-1 bg-[#646482]/15 relative group cursor-pointer">
        <div className="h-full transition-all duration-500"
          style={{ width: `${progress}%`, background: 'linear-gradient(90deg, #8B5CF6, #EC4899)' }} />
        <div className="absolute top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-white opacity-0 group-hover:opacity-100 transition-all shadow-lg -translate-x-1/2"
          style={{ left: `${progress}%` }} />
      </div>

      {/* ── Layout móvil ── */}
      <div className="flex flex-col px-4 py-2 gap-2 sm:hidden">

        {/* Fila 1: thumbnail + info + tiempo */}
        <div className="flex items-center gap-3">
          <div className="relative shrink-0">
            {track?.thumbnail
              ? <img src={track.thumbnail} alt="" className="rounded-lg object-cover" style={{ width: 40, height: 40 }} />
              : <div className="rounded-lg bg-[#1e1e30] flex items-center justify-center" style={{ width: 40, height: 40 }}>
                  <div className="w-4 h-4 rounded-full border-2 border-[#646482]/30" />
                </div>
            }
            {playing && track && (
              <div className="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-[#8B5CF6] flex items-center justify-center">
                <div className="w-1 h-1 rounded-full bg-white animate-ping absolute" />
                <div className="w-1 h-1 rounded-full bg-white" />
              </div>
            )}
          </div>

          <div className="min-w-0 flex-1">
            {track ? (
              <>
                <p className="text-[#DCDCEB] text-xs font-semibold truncate leading-tight">{track.title}</p>
                <p className="text-[#646482] text-xs truncate">{track.artist}</p>
                {state?.status_msg && (
                  <p className="text-[#8B5CF6] text-xs truncate animate-pulse">{state.status_msg}</p>
                )}
              </>
            ) : (
              <p className="text-[#646482] text-xs italic">Sin reproducir</p>
            )}
          </div>

          <span className="text-[#646482] text-xs tabular-nums shrink-0">
            {fmt(elapsed)}<span className="text-[#646482]/40"> / </span>{fmt(duration)}
          </span>
        </div>

        {/* Fila 2: controles centrados */}
        <div className="flex items-center justify-center gap-6 pb-1">
          <button onClick={() => cmd('prev')} disabled={!track}
            className="text-[#646482] hover:text-[#DCDCEB] transition disabled:opacity-20">
            <SkipBack size={18} fill="currentColor" />
          </button>

          <button onClick={togglePlay} disabled={!track}
            className="flex items-center justify-center rounded-full transition-all hover:scale-105 disabled:opacity-30"
            style={{ width: 40, height: 40, background: 'linear-gradient(135deg, #8B5CF6, #EC4899)', boxShadow: playing ? '0 0 16px rgba(139,92,246,0.5)' : '0 4px 12px rgba(0,0,0,0.4)' }}>
            {playing
              ? <Pause size={18} fill="white" color="white" />
              : <Play  size={18} fill="white" color="white" className="ml-0.5" />}
          </button>

          <button onClick={() => cmd('skip')} disabled={!track}
            className="text-[#646482] hover:text-[#DCDCEB] transition disabled:opacity-20">
            <SkipForward size={18} fill="currentColor" />
          </button>

          <button onClick={() => cmd('stop')} disabled={!track}
            className="text-[#646482] hover:text-[#EC4899] transition disabled:opacity-20">
            <Square size={14} fill="currentColor" />
          </button>

          <div className={`w-1.5 h-1.5 rounded-full ${connected ? 'bg-emerald-400' : 'bg-[#646482]'}`} />
        </div>
      </div>

      {/* ── Layout desktop ── */}
      <div className="hidden sm:flex items-center justify-between px-6 h-20">

        {/* Canción — izquierda */}
        <div className="flex items-center gap-3 w-72 min-w-0">
          <div className="relative shrink-0">
            {track?.thumbnail
              ? <img src={track.thumbnail} alt="" className="rounded-xl object-cover shadow-lg" style={{ width: 52, height: 52 }} />
              : <div className="rounded-xl bg-[#1e1e30] flex items-center justify-center" style={{ width: 52, height: 52 }}>
                  <div className="w-5 h-5 rounded-full border-2 border-[#646482]/30" />
                </div>
            }
            {playing && track && (
              <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-[#8B5CF6] flex items-center justify-center">
                <div className="w-1.5 h-1.5 rounded-full bg-white animate-ping absolute" />
                <div className="w-1.5 h-1.5 rounded-full bg-white" />
              </div>
            )}
          </div>
          <div className="min-w-0">
            {track ? (
              <>
                <p className="text-[#DCDCEB] text-sm font-semibold truncate leading-tight">{track.title}</p>
                <p className="text-[#646482] text-xs truncate mt-0.5">{track.artist}</p>
                {state?.status_msg && (
                  <p className="text-[#8B5CF6] text-xs truncate mt-0.5 animate-pulse">{state.status_msg}</p>
                )}
              </>
            ) : (
              <p className="text-[#646482] text-sm italic">Sin reproducir</p>
            )}
          </div>
        </div>

        {/* Controles — centro */}
        <div className="flex items-center gap-5">
          <button onClick={() => cmd('prev')} disabled={!track}
            className="text-[#646482] hover:text-[#DCDCEB] transition-all hover:scale-110 disabled:opacity-20 disabled:cursor-not-allowed">
            <SkipBack size={19} fill="currentColor" />
          </button>
          <button onClick={togglePlay} disabled={!track}
            className="relative flex items-center justify-center rounded-full transition-all hover:scale-105 disabled:opacity-30 disabled:cursor-not-allowed"
            style={{ width: 48, height: 48, background: 'linear-gradient(135deg, #8B5CF6, #EC4899)', boxShadow: playing ? '0 0 20px rgba(139,92,246,0.5)' : '0 4px 15px rgba(0,0,0,0.4)' }}>
            {playing
              ? <Pause size={20} fill="white" color="white" />
              : <Play  size={20} fill="white" color="white" className="ml-0.5" />}
          </button>
          <button onClick={() => cmd('skip')} disabled={!track}
            className="text-[#646482] hover:text-[#DCDCEB] transition-all hover:scale-110 disabled:opacity-20 disabled:cursor-not-allowed">
            <SkipForward size={19} fill="currentColor" />
          </button>
        </div>

        {/* Tiempo + stop — derecha */}
        <div className="flex items-center gap-4 w-72 justify-end">
          <div className="flex items-center gap-1.5 text-xs tabular-nums">
            <span className="text-[#DCDCEB]">{fmt(elapsed)}</span>
            <span className="text-[#646482]/50">/</span>
            <span className="text-[#646482]">{fmt(duration)}</span>
          </div>
          <div className="w-px h-4 bg-[#646482]/20" />
          <button onClick={() => cmd('stop')} disabled={!track}
            className="text-[#646482] hover:text-[#EC4899] transition-all hover:scale-110 disabled:opacity-20 disabled:cursor-not-allowed">
            <Square size={15} fill="currentColor" />
          </button>
          <div className={`w-1.5 h-1.5 rounded-full transition-colors ${connected ? 'bg-emerald-400' : 'bg-[#646482]'}`} />
        </div>
      </div>
    </footer>
  );
};

export default PlayerBar;