import { Search, Loader2 } from 'lucide-react';

const SearchBar = ({ busqueda, setBusqueda, cargando, manejarBusqueda }) => (
  <form onSubmit={manejarBusqueda} className="relative flex-1 max-w-xl">
    <div className="absolute left-4 top-1/2 -translate-y-1/2 text-[#646482]">
      {cargando
        ? <Loader2 size={16} className="animate-spin text-[#8B5CF6]" />
        : <Search size={16} />}
    </div>
    <input
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
);

export default SearchBar;