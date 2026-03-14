import { useState } from 'react';
import { CONFIG } from '../config.js';

export const useSearch = () => {
  const [busqueda, setBusqueda]     = useState('');
  const [resultados, setResultados] = useState([]);
  const [cargando, setCargando]     = useState(false);
  const [modalAbierto, setModal]    = useState(false);
  const [toast, setToast]           = useState(null);
  const [encolando, setEncolando]   = useState(null);

  const mostrarToast = (track) => {
    setToast(track);
    setTimeout(() => setToast(null), 3000);
  };

  const cerrarModal = () => {
    setModal(false);
    setResultados([]);
    setBusqueda('');
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

  return {
    busqueda, setBusqueda,
    resultados, cargando,
    modalAbierto, cerrarModal,
    manejarBusqueda, ponerEnCola,
    encolando, toast,
  };
};