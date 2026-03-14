import PlayerBar from './components/PlayerBar.jsx';
import ParticleBackground from './components/fondo.jsx';
import SearchBar from './components/Busqueda.jsx';
import PlaylistPanel from './components/Playlist.jsx';
import SearchModal from './components/busquedaM.jsx';
import { usePlaylist } from './hooks/usePlay.js';
import { useSearch } from './hooks/useSearch.js';
import { useEffect, useState } from 'react';

const GlitchLogo = () => {
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
    <>
      <style>{`
        @keyframes glitchA {
          0%   { clip-path: inset(0 0 90% 0); transform: translate(-3px, 0); }
          25%  { clip-path: inset(40% 0 40% 0); transform: translate(3px, 0); }
          50%  { clip-path: inset(70% 0 10% 0); transform: translate(-2px, 0); }
          75%  { clip-path: inset(20% 0 60% 0); transform: translate(2px, 0); }
          100% { clip-path: inset(0 0 90% 0); transform: translate(0); }
        }
        @keyframes glitchB {
          0%   { clip-path: inset(50% 0 20% 0); transform: translate(3px, 0); }
          33%  { clip-path: inset(10% 0 70% 0); transform: translate(-3px, 0); }
          66%  { clip-path: inset(80% 0 5% 0);  transform: translate(2px, 0); }
          100% { clip-path: inset(50% 0 20% 0); transform: translate(0); }
        }
        @keyframes logoPulse {
          0%,100% { transform: scale(1); }
          50%      { transform: scale(1.04); }
        }
        .logo-base   { animation: logoPulse 3s ease-in-out infinite; }
        .glitch-a    { animation: glitchA 0.4s steps(1) forwards; color: #EC4899; position: absolute; inset: 0; }
        .glitch-b    { animation: glitchB 0.4s steps(1) forwards; color: #8B5CF6; position: absolute; inset: 0; filter: brightness(1.5); }
      `}</style>

      <div className="relative select-none shrink-0 cursor-default"
        style={{ background: '#0C0C12', borderRadius: 10, padding: '6px 12px', border: '1px solid rgba(139,92,246,0.2)' }}>
        {/* Scanlines */}
        <div className="absolute inset-0 pointer-events-none rounded-[10px] overflow-hidden"
          style={{ backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(100,100,130,0.04) 2px, rgba(100,100,130,0.04) 4px)' }} />

        <span className="logo-base font-bold tracking-wider text-[#8B5CF6] relative z-10 text-base sm:text-xl"
          style={{ display: 'inline-block', fontFamily: 'monospace' }}>
          DeiTune
        </span>

        {glitch && <>
          <span className="glitch-a font-bold tracking-wider text-base sm:text-xl pointer-events-none"
            style={{ fontFamily: 'monospace' }}>DeiTune</span>
          <span className="glitch-b font-bold tracking-wider text-base sm:text-xl pointer-events-none"
            style={{ fontFamily: 'monospace' }}>DeiTune</span>
        </>}
      </div>
    </>
  );
};

const App = () => {
  const search = useSearch();
  const playlist = usePlaylist();

  return (
    <div className="h-screen bg-[#0C0C12] text-[#DCDCEB] overflow-hidden relative">
      <ParticleBackground />
      <div className="relative z-10 h-full flex flex-col">

        {/* Header */}
        <div className="shrink-0 px-6 pt-5 pb-4 border-b border-[#646482]/10 relative flex items-center"
          style={{ background: 'rgba(12,12,18,0.8)', backdropFilter: 'blur(20px)' }}>

          {/* Logo — esquina izquierda */}
          <GlitchLogo/>
          {/* Buscador — centrado */}
          <div className="w-full flex justify-center">
            <div className="w-full max-w-xl">
              <SearchBar {...search} />
            </div>
          </div>

        </div>

        {/* Playlist */}
        <div className="flex-1 overflow-y-auto pb-24">
          <PlaylistPanel {...playlist} />
        </div>

      </div>

      {/* Modal resultados */}
      <SearchModal {...search} onEncolar={async (track) => {
        await search.ponerEnCola(track);
      }} />

      <PlayerBar />
    </div>
  );
};

export default App;