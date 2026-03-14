import { useState, useRef, useEffect } from 'react';
import { Search, Loader2, CheckCircle2, X, Music, GripVertical, Play } from 'lucide-react';
import { CONFIG } from '../config.js';

// ── Toast ─────────────────────────────────────────────────────────────────────
const Toast = ({ toast }) => (
  <div className={`fixed top-6 right-6 z-50 transition-all duration-300 ${toast ? 'opacity-100 translate-y-0' : 'opacity-0 -translate-y-2 pointer-events-none'}`}>
    <div className={`flex items-center gap-3 px-4 py-3 rounded-xl shadow-2xl border text-sm max-w-xs
      ${toast?.error ? 'bg-[#1e1020] border-[#EC4899]/30 text-[#EC4899]' : 'bg-[#8B5CF6] border-[#8B5CF6] text-white'}`}>
      <CheckCircle2 size={16} className={toast?.error ? 'text-[#EC4899]' : 'text-white'} />
      <div className="min-w-0">
        <p className="font-medium truncate">{toast?.error ? toast.title : 'Agregado a la cola'}</p>
        {!toast?.error && <p className="text-purple-200 text-xs truncate">{toast?.title}</p>}
      </div>
    </div>
  </div>
);

// ── MainView ──────────────────────────────────────────────────────────────────
const MainView = () => {
  // Búsqueda
  const [busqueda, setBusqueda]     = useState('');
  const [resultados, setResultados] = useState([]);
  const [cargando, setCargando]     = useState(false);
  const [modalAbierto, setModal]    = useState(false);
  const [toast, setToast]           = useState(null);
  const [encolando, setEncolando]   = useState(null);

  // Playlist
  const [queue, setQueue]       = useState([]);
  const [current, setCurrent]   = useState(null);
  const [dragging, setDragging] = useState(null);
  const [dragOver, setDragOver] = useState(null);
  const wsRef    = useRef(null);
  const inputRef = useRef(null);

  // ── WebSocket + fetch inicial ───────────────────────────────────────────
  useEffect(() => {
    const fetchInitial = async () => {
      try {
        const [cr, qr] = await Promise.all([
          fetch(`${CONFIG.API}/current`),
          fetch(`${CONFIG.API}/queue`),
        ]);
        if (cr.ok) { const d = await cr.json(); setCurrent(d.current ?? null); }
        if (qr.ok) { const d = await qr.json(); setQueue(d ?? []); }
      } catch {}
    };
    fetchInitial();

    const connect = () => {
      const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(`${proto}//${window.location.host}/ws`);
      wsRef.current = ws;
      ws.onmessage = (e) => {
        try {
          const data = JSON.parse(e.data);
          setQueue(data.queue ?? []);
          setCurrent(data.current ?? null);
        } catch {}
      };
      ws.onclose = () => setTimeout(connect, 2000);
    };
    connect();
    return () => wsRef.current?.close();
  }, []);

  // ── Búsqueda ───────────────────────────────────────────────────────────
  const mostrarToast = (track) => {
    setToast(track);
    setTimeout(() => setToast(null), 3000);
  };

  const manejarBusqueda = async (e) => {
    e.preventDefault();
    if (busqueda.trim() === '') return;
    setCargando(true);
    setResultados([]);
    setModal(true);
    try {
      const r = await fetch(`${CONFIG.API}/results?query=${encodeURIComponent(busqueda)}`);
      if (!r.ok) throw new Error();
      setResultados(await r.json());
    } catch {
      mostrarToast({ error: true, title: 'No se pudo conectar' });
      setModal(false);
    } finally {
      setCargando(false);
    }
  };

  const cerrarModal = () => {
    setModal(false);
    setResultados([]);
    setBusqueda('');
  };

  const ponerEnCola = async (track) => {
    if (encolando) return;
    setEncolando(track.id);
    try {
      await fetch(`${CONFIG.API}/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: track.id }),
      });
      mostrarToast(track);
      cerrarModal();
    } catch {
      mostrarToast({ error: true, title: 'Error al encolar' });
    } finally {
      setEncolando(null);
    }
  };

  // ── Drag & drop ────────────────────────────────────────────────────────
  const cmd = async (endpoint, body) => {
    try {
      await fetch(`${CONFIG.API}/${endpoint}`, {
        method: 'POST',
        headers: body ? { 'Content-Type': 'application/json' } : {},
        body: body ? JSON.stringify(body) : undefined,
      });
    } catch {}
  };

  const onDragStart = (e, i) => { setDragging(i); e.dataTransfer.effectAllowed = 'move'; };
  const onDragEnter = (e, i) => { e.preventDefault(); if (i !== dragging) setDragOver(i); };
  const onDragOver  = (e)    => e.preventDefault();
  const onDragEnd   = ()     => { setDragging(null); setDragOver(null); };

  const onDrop = async (e, toIndex) => {
    e.preventDefault();
    if (dragging === null || dragging === toIndex) return;
    const next = [...queue];
    const [item] = next.splice(dragging, 1);
    next.splice(toIndex, 0, item);
    setQueue(next);
    await cmd('move', { from: dragging, to: toIndex });
    setDragging(null);
    setDragOver(null);
  };

  return (
    <div className="flex flex-col h-full">
      <Toast toast={toast} />

      {/* ── Barra de búsqueda fija arriba ───────────────────────────── */}
      <div className="shrink-0 px-6 pt-6 pb-4 border-b border-[#646482]/10"
        style={{ background: 'rgba(12,12,18,0.8)', backdropFilter: 'blur(20px)' }}>
        <div className="max-w-2xl mx-auto flex items-center gap-3">
          <span className="text-[#8B5CF6] font-bold text-lg tracking-wider shrink-0">DeiTune</span>
          <form onSubmit={manejarBusqueda} className="relative flex-1">
            <div className="absolute left-4 top-1/2 -translate-y-1/2 text-[#646482]">
              {cargando
                ? <Loader2 size={16} className="animate-spin text-[#8B5CF6]" />
                : <Search size={16} />}
            </div>
            <input
              ref={inputRef}
              type="text"
              placeholder="Buscar canción..."
              value={busqueda}
              onChange={(e) => setBusqueda(e.target.value)}
              disabled={cargando}
              className="w-full bg-[#161620] border border-[#646482]/20 text-[#DCDCEB] pl-10 pr-28 py-2.5 rounded-xl
                focus:outline-none focus:border-[#8B5CF6]/50 transition placeholder:text-[#646482] text-sm"
            />
            <button type="submit" disabled={cargando || busqueda.trim() === ''}
              className="absolute right-1.5 top-1/2 -translate-y-1/2 bg-[#8B5CF6] hover:bg-[#7c4dff]
                disabled:opacity-40 text-white px-4 py-1.5 rounded-lg text-sm font-medium transition">
              Buscar
            </button>
          </form>
        </div>
      </div>

      {/* ── Playlist — ocupa el resto de la pantalla ────────────────── */}
      <div className="flex-1 overflow-y-auto pb-24">
        <div className="max-w-2xl mx-auto px-6 pt-6">

          {/* Canción actual */}
          {current && (
            <div className="mb-6">
              <p className="text-xs text-[#8B5CF6] font-medium uppercase tracking-widest mb-2">Sonando ahora</p>
              <div className="flex items-center gap-3 bg-[#1e1e2e] rounded-xl p-3 border border-[#8B5CF6]/20">
                {current.thumbnail
                  ? <img src={current.thumbnail} alt="" className="w-12 h-12 rounded-lg object-cover shrink-0" />
                  : <div className="w-12 h-12 rounded-lg bg-[#2a2a40] shrink-0 flex items-center justify-center"><Music size={16} className="text-[#646482]" /></div>
                }
                <div className="min-w-0 flex-1">
                  <p className="text-[#DCDCEB] text-sm font-medium truncate">{current.title}</p>
                  <p className="text-[#646482] text-xs truncate">{current.artist}</p>
                </div>
                <div className="flex items-end gap-0.5 h-4 shrink-0">
                  {[0, 0.2, 0.4].map((d, i) => (
                    <div key={i} className="w-0.5 bg-[#8B5CF6] rounded-full animate-bounce"
                      style={{ height: '100%', animationDelay: `${d}s`, animationDuration: '0.7s' }} />
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Cola */}
          <div className="flex items-center justify-between mb-3">
            <p className="text-xs text-[#646482] uppercase tracking-widest">
              {queue.length > 0 ? `${queue.length} en cola` : 'Cola vacía'}
            </p>
          </div>

          {queue.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 text-center">
              <div className="w-14 h-14 rounded-full bg-[#1e1e2e] flex items-center justify-center mb-4">
                <Music size={24} className="text-[#646482]" />
              </div>
              <p className="text-[#DCDCEB] font-medium mb-1">La cola está vacía</p>
              <p className="text-[#646482] text-sm">Busca una canción para agregarla</p>
            </div>
          ) : (
            <ul className="flex flex-col gap-1">
              {queue.map((track, i) => (
                <li key={track.id + i} draggable
                  onDragStart={(e) => onDragStart(e, i)}
                  onDragEnter={(e) => onDragEnter(e, i)}
                  onDragOver={onDragOver}
                  onDrop={(e) => onDrop(e, i)}
                  onDragEnd={onDragEnd}
                  className={`flex items-center gap-3 rounded-xl px-3 py-2.5 group transition-all select-none
                    ${dragging === i ? 'opacity-30' : 'opacity-100'}
                    ${dragOver === i ? 'bg-[#8B5CF6]/10 border border-[#8B5CF6]/30' : 'bg-[#161620] border border-transparent hover:border-[#646482]/20'}`}>
                  <div className="text-[#646482] cursor-grab opacity-0 group-hover:opacity-100 transition shrink-0">
                    <GripVertical size={16} />
                  </div>
                  <span className="text-[#646482] text-xs w-5 text-right shrink-0 tabular-nums group-hover:hidden">{i + 1}</span>
                  {track.thumbnail
                    ? <img src={track.thumbnail} alt="" className="w-10 h-10 rounded-lg object-cover shrink-0" />
                    : <div className="w-10 h-10 rounded-lg bg-[#2a2a40] shrink-0 flex items-center justify-center"><Music size={12} className="text-[#646482]" /></div>
                  }
                  <div className="min-w-0 flex-1">
                    <p className="text-[#DCDCEB] text-sm truncate leading-tight">{track.title}</p>
                    <p className="text-[#646482] text-xs truncate">{track.artist}</p>
                  </div>
                  <span className="text-[#646482] text-xs tabular-nums shrink-0">{track.duration}</span>
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition shrink-0">
                    <button onClick={() => cmd('play_at', { index: i })}
                      className="p-1.5 rounded-lg text-[#646482] hover:text-[#8B5CF6] hover:bg-[#8B5CF6]/10 transition">
                      <Play size={14} fill="currentColor" />
                    </button>
                    <button onClick={() => cmd('remove', { index: i })}
                      className="p-1.5 rounded-lg text-[#646482] hover:text-[#EC4899] hover:bg-[#EC4899]/10 transition">
                      <X size={14} />
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* ── Modal resultados ────────────────────────────────────────── */}
      {/* Overlay */}
      <div onClick={cerrarModal}
        className={`fixed inset-0 z-40 transition-all duration-200 ${modalAbierto ? 'bg-black/60 pointer-events-auto' : 'bg-transparent pointer-events-none'}`} />

      {/* Panel modal */}
      <div className={`fixed top-[72px] left-1/2 -translate-x-1/2 w-full max-w-2xl z-50 px-6 transition-all duration-200
        ${modalAbierto ? 'opacity-100 translate-y-0' : 'opacity-0 -translate-y-3 pointer-events-none'}`}>
        <div className="bg-[#1a1a28] border border-[#646482]/20 rounded-2xl shadow-2xl overflow-hidden max-h-[70vh] flex flex-col">

          {/* Header modal */}
          <div className="flex items-center justify-between px-4 py-3 border-b border-[#646482]/10 shrink-0">
            <p className="text-xs text-[#646482]">
              {cargando ? 'Buscando...' : `${resultados.length} resultados`}
            </p>
            <button onClick={cerrarModal} className="text-[#646482] hover:text-[#DCDCEB] transition">
              <X size={16} />
            </button>
          </div>

          {/* Resultados */}
          <div className="overflow-y-auto">
            {cargando ? (
              <div className="flex flex-col gap-2 p-4">
                {[...Array(4)].map((_, i) => (
                  <div key={i} className="h-14 rounded-xl bg-[#161620] animate-pulse" style={{ opacity: 1 - i * 0.2 }} />
                ))}
              </div>
            ) : (
              <div className="flex flex-col divide-y divide-[#646482]/10">
                {resultados.map((cancion) => (
                  <button key={cancion.id}
                    onClick={() => ponerEnCola(cancion)}
                    disabled={encolando === cancion.id}
                    className={`flex items-center gap-3 px-4 py-3 hover:bg-[#8B5CF6]/10 transition text-left w-full
                      ${encolando === cancion.id ? 'opacity-50' : ''}`}>
                    {cancion.thumbnail
                      ? <img src={cancion.thumbnail} alt="" className="w-10 h-10 rounded-lg object-cover shrink-0" />
                      : <div className="w-10 h-10 rounded-lg bg-[#2a2a40] shrink-0 flex items-center justify-center"><Music size={12} className="text-[#646482]" /></div>
                    }
                    <div className="min-w-0 flex-1">
                      <p className="text-[#DCDCEB] text-sm font-medium truncate">{cancion.title}</p>
                      <p className="text-[#646482] text-xs truncate">{cancion.artist}</p>
                    </div>
                    <span className="text-[#646482] text-xs shrink-0">{cancion.duration}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default MainView;