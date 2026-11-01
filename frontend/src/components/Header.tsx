import React from 'react';
import { useStore, type ViewMode } from '../store';

const monthNames = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
];

const rangeTitle = (view: string, cursor: Date): string => {
  if (view === 'month') return `${monthNames[cursor.getMonth()]} ${cursor.getFullYear()}`;
  if (view === 'day') {
    return cursor.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
  }
  const end = new Date(cursor);
  end.setDate(cursor.getDate() + (view === 'agenda' ? 3 : 6));
  const fmt = (d: Date) => d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  return `${fmt(cursor)} – ${fmt(end)}, ${cursor.getFullYear()}`;
};

export const Header: React.FC = () => {
  const { viewMode, cursor, setViewMode, goToday, goPrev, goNext } = useStore();

  const views: { id: ViewMode; label: string }[] = [
    { id: 'month', label: 'Month' },
    { id: 'week', label: 'Week' },
    { id: 'day', label: 'Day' },
    { id: 'agenda', label: 'Agenda' },
  ];

  return (
    <header style={{
      padding: '8px 12px', backgroundColor: '#fff', borderBottom: '1px solid #dadce0',
      display: 'flex', alignItems: 'center', gap: 8,
    }}>
      <button onClick={goToday} style={{ padding: '4px 12px', border: '1px solid #dadce0', borderRadius: 4, cursor: 'pointer' }}>
        Today
      </button>
      <button onClick={goPrev} aria-label="Previous" style={{ border: 'none', background: 'none', cursor: 'pointer', fontSize: 18 }}>‹</button>
      <button onClick={goNext} aria-label="Next" style={{ border: 'none', background: 'none', cursor: 'pointer', fontSize: 18 }}>›</button>
      <span style={{ marginLeft: 8, fontSize: 18, color: '#3c4043' }}>{rangeTitle(viewMode, cursor)}</span>
      <div style={{ marginLeft: 'auto', display: 'flex', border: '1px solid #dadce0', borderRadius: 4, overflow: 'hidden' }}>
        {views.map((v) => (
          <button
            key={v.id}
            onClick={() => setViewMode(v.id)}
            style={{
              padding: '4px 12px', border: 'none', cursor: 'pointer',
              background: viewMode === v.id ? '#e8f0fe' : '#fff',
              color: viewMode === v.id ? '#1a73e8' : '#3c4043',
              fontWeight: viewMode === v.id ? 600 : 400,
            }}
          >
            {v.label}
          </button>
        ))}
      </div>
    </header>
  );
};
