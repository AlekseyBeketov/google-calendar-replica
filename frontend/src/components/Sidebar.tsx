import React, { useState } from 'react';
import { useStore, ymd } from '../store';

const PALETTE = ['#d93025', '#e37400', '#f0ad00', '#0b8043', '#3f51b5', '#1a73e8', '#8e24aa', '#d81b60'];

const MiniCalendar: React.FC = () => {
  const { cursor, setCursor, events } = useStore();
  const now = new Date();
  const todayKey = ymd(now);
  const cursorKey = ymd(cursor);
  const busy = new Set(events.map((e) => e.occurrence_date));

  const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
  const shift = (first.getDay() + 6) % 7;
  const cells: (Date | null)[] = Array(shift).fill(null);
  for (let i = 0; i < 31; i++) {
    const d = new Date(first);
    d.setDate(first.getDate() + i);
    if (d.getMonth() === cursor.getMonth()) cells.push(d);
  }

  return (
    <div style={{ fontSize: 11 }}>
      <div style={{ textAlign: 'center', fontWeight: 600, marginBottom: 4 }}>
        {cursor.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: 1 }}>
        {['M', 'T', 'W', 'T', 'F', 'S', 'S'].map((d, i) => (
          <div key={i} style={{ textAlign: 'center', color: '#70757a' }}>{d}</div>
        ))}
        {cells.map((d, i) => {
          if (!d) return <div key={`e${i}`} />;
          const key = ymd(d);
          const isToday = key === todayKey;
          const isCursor = key === cursorKey;
          return (
            <div
              key={key}
              onClick={() => setCursor(d)}
              style={{
                textAlign: 'center', padding: '2px 0', cursor: 'pointer', borderRadius: '50%',
                background: isCursor ? '#d3e3fd' : isToday ? '#1a73e8' : 'transparent',
                color: isCursor || isToday ? '#fff' : '#3c4043',
              }}
            >
              {d.getDate()}
              {busy.has(key) && !isToday && (
                <span style={{ display: 'block', width: 4, height: 4, borderRadius: '50%', background: '#1a73e8', margin: '1px auto 0' }} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export const Sidebar: React.FC = () => {
  const { calendars, toggleCalendar, addCalendar, setCursor } = useStore();
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState('');
  const [color, setColor] = useState(PALETTE[4]);

  const submitCalendar = async () => {
    if (!name.trim()) return;
    await addCalendar({ name: name.trim(), color });
    setName('');
    setAdding(false);
  };

  return (
    <aside style={{ width: 210, padding: '8px', borderRight: '1px solid #dadce0', flexShrink: 0 }}>
      <button
        onClick={() => setCursor(new Date())}
        style={{ border: '1px solid #dadce0', borderRadius: 4, padding: '4px 10px', cursor: 'pointer', marginBottom: 8 }}
      >
        Today
      </button>
      <MiniCalendar />
      <h4 style={{ fontSize: 13, color: '#5f6368', margin: '12px 0 4px' }}>My calendars</h4>
      <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
        {calendars.map((c) => (
          <li key={c.id} style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '3px 0' }}>
            <input
              type="checkbox"
              checked={c.visible}
              onChange={(e) => void toggleCalendar(c.id, e.target.checked)}
            />
            <span style={{ width: 10, height: 10, borderRadius: '50%', background: c.color, flexShrink: 0 }} />
            <span style={{ fontSize: 13, color: '#3c4043' }}>{c.name}</span>
          </li>
        ))}
      </ul>
      {adding ? (
        <div style={{ marginTop: 6, display: 'flex', flexDirection: 'column', gap: 4 }}>
          <input
            autoFocus
            value={name}
            placeholder="Calendar name"
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') void submitCalendar(); if (e.key === 'Escape') setAdding(false); }}
            style={{ padding: '2px 6px', fontSize: 13 }}
          />
          <div style={{ display: 'flex', gap: 4 }}>
            {PALETTE.map((p) => (
              <div
                key={p}
                onClick={() => setColor(p)}
                style={{
                  width: 14, height: 14, borderRadius: '50%', background: p, cursor: 'pointer',
                  outline: color === p ? '2px solid #1a73e8' : 'none', outlineOffset: 1,
                }}
              />
            ))}
          </div>
          <div style={{ display: 'flex', gap: 4 }}>
            <button onClick={() => void submitCalendar()}>Add</button>
            <button onClick={() => setAdding(false)}>Cancel</button>
          </div>
        </div>
      ) : (
        <button onClick={() => setAdding(true)} style={{ marginTop: 6, fontSize: 13, border: 'none', background: 'none', color: '#1a73e8', cursor: 'pointer' }}>
          + Create calendar
        </button>
      )}
    </aside>
  );
};
