import { useState, useEffect, useRef } from 'react';
import { Search, ListMusic, ChevronLeft } from 'lucide-react';

const Sidebar = ({ vistaActual, setVistaActual }) => {
  const [collapsed, setCollapsed] = useState(false);
  const [glitch, setGlitch] = useState(false);

  useEffect(() => {
    const loop = () => {
      const delay = 3000 + Math.random() * 4000;
      setTimeout(() => {
        setGlitch(true);
        setTimeout(() => setGlitch(false), 400);
        loop();
      }, delay);
    };
    loop();
  }, []);

  return (
    <aside className={`relative bg-[#161620] flex flex-col gap-6 border-r border-[#646482]/10 z-20 shadow-xl transition-all duration-300 ${collapsed ? 'w-16 p-3' : 'w-52 p-6'}`}>
      <style>{`
        @keyframes glitchShift {
          0%   { clip-path: inset(0 0 95% 0); transform: translate(-3px, 0); opacity: 0.8; }
          20%  { clip-path: inset(30% 0 50% 0); transform: translate(3px, 0); }
          40%  { clip-path: inset(60% 0 20% 0); transform: translate(-2px, 0); }
          60%  { clip-path: inset(10% 0 80% 0); transform: translate(2px, 0); }
          80%  { clip-path: inset(80% 0 5% 0);  transform: translate(-1px, 0); }
          100% { clip-path: inset(0 0 95% 0); transform: translate(0); opacity: 0; }
        }
        @keyframes glitchShift2 {
          0%   { clip-path: inset(50% 0 30% 0); transform: translate(3px, 0); opacity: 0.6; }
          25%  { clip-path: inset(20% 0 70% 0); transform: translate(-3px, 0); }
          50%  { clip-path: inset(70% 0 10% 0); transform: translate(2px, 0); }
          75%  { clip-path: inset(40% 0 40% 0); transform: translate(-2px, 0); }
          100% { clip-path: inset(50% 0 30% 0); transform: translate(0); opacity: 0; }
        }
        .glitch-layer-1 {
          animation: glitchShift 0.4s steps(1) forwards;
          color: #EC4899;
        }
        .glitch-layer-2 {
          animation: glitchShift2 0.4s steps(1) forwards;
          color: #8B5CF6;
        }
      `}</style>

      {/* Botón colapsar */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="absolute -right-3 top-8 w-6 h-6 rounded-full bg-[#8B5CF6] flex items-center justify-center shadow-lg hover:bg-[#7c4dff] transition-all z-10"
      >
        <ChevronLeft size={14} className={`text-white transition-transform duration-300 ${collapsed ? 'rotate-180' : ''}`} />
      </button>

      {/* Logo */}
      {!collapsed ? (
        <div className="relative select-none">
          <h1 className="text-[#8B5CF6] font-bold text-2xl tracking-wider whitespace-nowrap">
            DeiTune
          </h1>
          {/* Capas glitch — solo visibles durante el efecto */}
          {glitch && <>
            <h1 className="glitch-layer-1 font-bold text-2xl tracking-wider whitespace-nowrap absolute inset-0">
              DeiTune
            </h1>
            <h1 className="glitch-layer-2 font-bold text-2xl tracking-wider whitespace-nowrap absolute inset-0">
              DeiTune
            </h1>
          </>}
          <p className="text-[#646482] text-xs font-mono tracking-widest mt-0.5">v2.Latin.H0T</p>
        </div>
      ) : (
        <div className="w-8 h-8 rounded-lg bg-[#8B5CF6] flex items-center justify-center mx-auto mt-1">
          <span className="text-white font-bold text-sm">D</span>
        </div>
      )}

      {/* Nav */}
      <nav className="flex flex-col gap-3">
        <button
          onClick={() => setVistaActual('buscar')}
          title="Buscador"
          className={`flex items-center gap-3 p-3 rounded-lg transition-all font-semibold
            ${collapsed ? 'justify-center' : ''}
            ${vistaActual === 'buscar'
              ? 'bg-[#8B5CF6] text-white shadow-md'
              : 'text-[#646482] hover:bg-[#1E1E32] hover:text-[#DCDCEB]'}`}
        >
          <Search size={20} className="shrink-0" />
          {!collapsed && <span>Buscador</span>}
        </button>

        <button
          onClick={() => setVistaActual('playlist')}
          title="Mi Playlist"
          className={`flex items-center gap-3 p-3 rounded-lg transition-all font-semibold
            ${collapsed ? 'justify-center' : ''}
            ${vistaActual === 'playlist'
              ? 'bg-[#8B5CF6] text-white shadow-md'
              : 'text-[#646482] hover:bg-[#1E1E32] hover:text-[#DCDCEB]'}`}
        >
          <ListMusic size={20} className="shrink-0" />
          {!collapsed && <span>Mi Playlist</span>}
        </button>
      </nav>
    </aside>
  );
};

export default Sidebar;