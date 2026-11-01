import React from 'react';
import { useStore } from '../store';

export const CalendarView: React.FC = () => {
  const { viewMode, events, cursor } = useStore();

  // Simple placeholder for each view
  if (viewMode === 'month') {
    const start = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const days: JSX.Element[] = [];
    for (let i = 0; i < 42; i++) {
      const d = new Date(start);
      d.setDate(start.getDate() + i);
      days.push(
        <div key={i} style={{ border: '1px solid #eee', padding: '4px', height: '80px' }}>
          {d.getDate()}
        </div>
      );
    }
    return (
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '2px' }}>
        {days}
      </div>
    );
  }
  // For other views, just show a placeholder text
  return (
    <div style={{ flex: 1, padding: '1rem' }}>
      <h3>{viewMode.charAt(0).toUpperCase() + viewMode.slice(1)} view (TODO)</h3>
    </div>
  );
};
