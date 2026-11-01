import React, { useState } from 'react';
import { useStore } from '../store';
import { invoke } from '@tauri-apps/api/core';
import type { EventInput } from '../types';

export const EventModal: React.FC = () => {
  const { calendars, cursor } = useStore();
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [allDay, setAllDay] = useState(false);
  const [calendarId, setCalendarId] = useState('');

  const createEvent = async () => {
    if (!title.trim() || !calendarId) return;
    const input: EventInput = {
      calendar_id: calendarId,
      title: title.trim(),
      all_day: allDay,
      tz_id: 'Europe/Moscow',
    };
    if (!allDay) {
      const start = new Date(cursor);
      const end = new Date(cursor.getTime() + 60 * 60 * 1000);
      input.start_utc = start.toISOString();
      input.end_utc = end.toISOString();
    }
    await invoke('create_event', { input });
    setOpen(false);
    setTitle('');
  };

  if (!open) {
    return (
      <button
        style={{ position: 'fixed', bottom: '1rem', right: '1rem', zIndex: 100 }}
        onClick={() => setOpen(true)}
      >
        + Create
      </button>
    );
  }

  return (
    <div style={{
      position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.3)',
      display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 200,
    }}>
      <div style={{ background: '#fff', padding: '1rem', borderRadius: '8px', minWidth: '320px' }}>
        <h3>New event</h3>
        <input
          placeholder="Title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          style={{ width: '100%', marginBottom: '0.5rem' }}
        />
        <label style={{ display: 'block', marginBottom: '0.5rem' }}>
          <input type="checkbox" checked={allDay} onChange={(e) => setAllDay(e.target.checked)} /> All day
        </label>
        <select value={calendarId} onChange={(e) => setCalendarId(e.target.value)} style={{ width: '100%', marginBottom: '0.75rem' }}>
          <option value="">Select calendar…</option>
          {calendars.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
          <button onClick={() => setOpen(false)}>Cancel</button>
          <button onClick={createEvent} disabled={!title.trim() || !calendarId}>Save</button>
        </div>
      </div>
    </div>
  );
};
