import { X, Music, CheckCircle2 } from 'lucide-react';

const SearchModal = ({ modalAbierto, cerrarModal, resultados, cargando, encolando, ponerEnCola, toast }) => (
  <>
    {/* Toast */}
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

    {/* Overlay */}
    <div onClick={cerrarModal}
      className={`fixed inset-0 z-40 transition-all duration-200 ${modalAbierto ? 'bg-black/60 pointer-events-auto' : 'bg-transparent pointer-events-none'}`} />

    {/* Panel */}
    <div className={`fixed top-[72px] left-1/2 -translate-x-1/2 w-full max-w-2xl z-50 px-6 transition-all duration-200
      ${modalAbierto ? 'opacity-100 translate-y-0' : 'opacity-0 -translate-y-3 pointer-events-none'}`}>
      <div className="bg-[#1a1a28] border border-[#646482]/20 rounded-2xl shadow-2xl overflow-hidden max-h-[70vh] flex flex-col">

        <div className="flex items-center justify-between px-4 py-3 border-b border-[#646482]/10 shrink-0">
          <p className="text-xs text-[#646482]">
            {cargando ? 'Buscando...' : `${resultados.length} resultados`}
          </p>
          <button onClick={cerrarModal} className="text-[#646482] hover:text-[#DCDCEB] transition">
            <X size={16} />
          </button>
        </div>

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
  </>
);

export default SearchModal;