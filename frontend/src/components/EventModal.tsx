import React, { useEffect, useState } from 'react';
import { useStore, ymd } from '../store';
import { invoke } from '@tauri-apps/api/core';
import type { EventDetail, EventInput, EventPatch, Scope } from '../types';

const TZ = 'Europe/Moscow';

const REPEATS: { id: string; label: string }[] = [
  { id: '', label: 'Does not repeat' },
  { id: 'FREQ=DAILY', label: 'Daily' },
  { id: 'FREQ=WEEKLY', label: 'Weekly' },
  { id: 'FREQ=MONTHLY', label: 'Monthly' },
  { id: 'FREQ=YEARLY', label: 'Yearly' },
];

/** ISO-UTC string -> value for <input type="datetime-local"> (local time). */
const toLocalInput = (iso?: string): string => {
  if (!iso) return '';
  const d = new Date(iso);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
};

const localInputToUtc = (v: string): string => (v ? new Date(v).toISOString() : '');

/** Display end for all-day: stored end is exclusive, input shows inclusive. */
const inclusiveEnd = (dateStr?: string): string => {
  if (!dateStr) return '';
  const d = new Date(dateStr + 'T00:00:00');
  d.setDate(d.getDate() - 1);
  return ymd(d);
};
const exclusiveEnd = (v: string): string => {
  const d = new Date(v + 'T00:00:00');
  d.setDate(d.getDate() + 1);
  return ymd(d);
};

const parseFreq = (rrule?: string | null): string => {
  if (!rrule) return '';
  const m = /FREQ=(DAILY|WEEKLY|MONTHLY|YEARLY)/.exec(rrule.toUpperCase());
  return m ? `FREQ=${m[1]}` : '';
};

export const EventModal: React.FC = () => {
  const { modal, closeModal, calendars, createEvent, updateEvent, deleteEvent, cursor } = useStore();
  const editing = !!modal.eventId;

  const [title, setTitle] = useState('');
  const [allDay, setAllDay] = useState(false);
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [startTime, setStartTime] = useState('09:00');
  const [endTime, setEndTime] = useState('10:00');
  const [repeat, setRepeat] = useState('');
  const [calendarId, setCalendarId] = useState('');
  const [description, setDescription] = useState('');
  const [location, setLocation] = useState('');
  const [scope, setScope] = useState<Scope>('all');
  const [loaded, setLoaded] = useState(false);
  const [isSeries, setIsSeries] = useState(false);

  useEffect(() => {
    if (!modal.open) { setLoaded(false); return; }
    const slot = modal.slot ?? cursor;
    setStartDate(ymd(slot));
    setEndDate(ymd(slot));
    setStartTime('09:00');
    setEndTime('10:00');
    setTitle('');
    setAllDay(false);
    setRepeat('');
    setDescription('');
    setLocation('');
    setScope('all');
    setCalendarId(calendars[0]?.id ?? '');

    if (modal.eventId) {
      invoke<EventDetail>('get_event', { id: modal.eventId }).then((d) => {
        const e = d.event;
        setTitle(e.title);
        setAllDay(e.all_day);
        setCalendarId(e.calendar_id);
        setDescription(e.description);
        setLocation(e.location);
        setRepeat(parseFreq(e.rrule));
        setIsSeries(d.is_series);
        if (e.all_day) {
          setStartDate(e.start_date ?? '');
          setEndDate(inclusiveEnd(e.end_date));
        } else {
          setStartDate(e.start_utc ? e.start_utc.slice(0, 10) : '');
          setEndDate(e.start_utc ? e.start_utc.slice(0, 10) : '');
          setStartTime(toLocalInput(e.start_utc ?? undefined).slice(11) || '09:00');
          setEndTime(toLocalInput(e.end_utc ?? undefined).slice(11) || '10:00');
        }
        setLoaded(true);
      });
    } else {
      setLoaded(true);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [modal.open, modal.eventId, modal.slot]);

  const save = async () => {
    if (!title.trim() || !calendarId) return;
    if (editing && modal.eventId) {
      const patch: EventPatch = {
        title: title.trim(),
        all_day: allDay,
        description,
        location,
        rrule: repeat || undefined,
        tz_id: TZ,
      };
      if (allDay) {
        patch.start_date = startDate;
        patch.end_date = exclusiveEnd(endDate || startDate);
        patch.start_utc = undefined;
        patch.end_utc = undefined;
      } else {
        const s = new Date(`${startDate}T${startTime}`);
        const e = new Date(`${endDate || startDate}T${endTime}`);
        patch.start_utc = s.toISOString();
        patch.end_utc = e.toISOString();
        patch.start_date = undefined;
        patch.end_date = undefined;
      }
      await updateEvent(modal.eventId, patch, scope, scope === 'this' ? modal.occurrenceDate : undefined);
    } else {
      const input: EventInput = {
        calendar_id: calendarId,
        title: title.trim(),
        all_day: allDay,
        tz_id: TZ,
        description,
        location,
        rrule: repeat || undefined,
      };
      if (allDay) {
        input.start_date = startDate;
        input.end_date = exclusiveEnd(endDate || startDate);
      } else {
        const s = new Date(`${startDate}T${startTime}`);
        const e = new Date(`${endDate || startDate}T${endTime}`);
        input.start_utc = s.toISOString();
        input.end_utc = e.toISOString();
      }
      await createEvent(input);
    }
    closeModal();
  };

  const remove = async () => {
    if (!modal.eventId) return;
    await deleteEvent(modal.eventId, scope, scope === 'this' ? modal.occurrenceDate : undefined);
    closeModal();
  };

  if (!modal.open) return null;

  return (
    <div style={{
      position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.35)',
      display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 200,
    }} onClick={closeModal}>
      <div
        onClick={(e) => e.stopPropagation()}
        style={{ background: '#fff', padding: 20, borderRadius: 8, minWidth: 380, boxShadow: '0 8px 30px rgba(0,0,0,0.25)' }}
      >
        <h3 style={{ margin: '0 0 12px', color: '#3c4043' }}>
          {editing ? 'Edit event' : 'New event'}
        </h3>

        <input
          autoFocus
          placeholder="Add title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', fontSize: 15, marginBottom: 10, border: '1px solid #dadce0', borderRadius: 4 }}
        />

        <label style={{ display: 'block', marginBottom: 10, fontSize: 13 }}>
          <input type="checkbox" checked={allDay} onChange={(e) => setAllDay(e.target.checked)} /> All day
        </label>

        {!allDay && (
          <div style={{ display: 'flex', gap: 8, marginBottom: 10, alignItems: 'center', fontSize: 13 }}>
            <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
            <input type="time" value={startTime} onChange={(e) => setStartTime(e.target.value)} />
            <span>–</span>
            <input type="date" value={endDate || startDate} onChange={(e) => setEndDate(e.target.value)} />
            <input type="time" value={endTime} onChange={(e) => setEndTime(e.target.value)} />
          </div>
        )}
        {allDay && (
          <div style={{ display: 'flex', gap: 8, marginBottom: 10, alignItems: 'center', fontSize: 13 }}>
            <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
            <span>–</span>
            <input type="date" value={endDate || startDate} onChange={(e) => setEndDate(e.target.value)} />
          </div>
        )}

        <select
          value={repeat}
          onChange={(e) => setRepeat(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', marginBottom: 10 }}
        >
          {REPEATS.map((r) => <option key={r.id} value={r.id}>{r.label}</option>)}
        </select>

        <select
          value={calendarId}
          onChange={(e) => setCalendarId(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', marginBottom: 10 }}
        >
          {calendars.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </select>

        <textarea
          placeholder="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', marginBottom: 10, minHeight: 48, resize: 'vertical' }}
        />

        <input
          placeholder="Location"
          value={location}
          onChange={(e) => setLocation(e.target.value)}
          style={{ width: '100%', padding: '6px 8px', marginBottom: 10 }}
        />

        {editing && isSeries && (
          <label style={{ display: 'block', marginBottom: 10, fontSize: 13 }}>
            Applies to:{' '}
            <select value={scope} onChange={(e) => setScope(e.target.value as Scope)}>
              <option value="this">This event</option>
              <option value="this_and_following">This and following</option>
              <option value="all">All events</option>
            </select>
          </label>
        )}

        <div style={{ display: 'flex', gap: 8, justifyContent: 'space-between' }}>
          {editing ? (
            <button onClick={() => void remove()} style={{ color: '#d93025', border: 'none', background: 'none', cursor: 'pointer' }}>
              Delete
            </button>
          ) : <span />}
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={closeModal}>Cancel</button>
            <button
              onClick={() => void save()}
              disabled={!loaded || !title.trim() || !calendarId}
              style={{ background: '#1a73e8', color: '#fff', border: 'none', padding: '6px 16px', borderRadius: 4, cursor: 'pointer' }}
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
