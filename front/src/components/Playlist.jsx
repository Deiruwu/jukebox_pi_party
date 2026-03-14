import { Music, GripVertical, Play, X } from 'lucide-react';

const PlaylistPanel = ({ queue, current, cmd, dragging, dragOver, onDragStart, onDragEnter, onDragOver, onDragEnd, onDrop }) => (
  <div className="max-w-2xl mx-auto px-6 pt-6">

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

    <p className="text-xs text-[#646482] uppercase tracking-widest mb-3">
      {queue.length > 0 ? `${queue.length} en cola` : 'Cola vacía'}
    </p>

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
);

export default PlaylistPanel;
