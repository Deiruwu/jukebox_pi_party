const TrackCard = ({ track }) => {
  return (
   
   <div className="bg-[#161620] p-4 rounded-lg flex items-center gap-4"> 
      
      <img src={track.thumbnail} alt="Portada" className="w-16 h-16 rounded shadow-lg" />

      <div className="flex-1">
        <h1 className="font-bold text-#DCDCEB"> {track.title} </h1>
        <h3 className="text-sm text-#646482">{track.artist}</h3>
        <h5 className="text-xs text-#646482">({track.album})</h5>
        
      </div>

      <h3 className="text-sm text-[#DCDCEB]">{track.duration}</h3>
    </div>
  );
};


export default TrackCard;