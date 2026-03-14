import { useState, useEffect, useRef } from 'react';
import { CONFIG } from '../config.js';

export const usePlaylist = () => {
  const [queue, setQueue]       = useState([]);
  const [current, setCurrent]   = useState(null);
  const [dragging, setDragging] = useState(null);
  const [dragOver, setDragOver] = useState(null);
  const wsRef = useRef(null);

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

  return {
    queue, current, cmd,
    dragging, dragOver,
    onDragStart, onDragEnter, onDragOver, onDragEnd, onDrop,
  };
};