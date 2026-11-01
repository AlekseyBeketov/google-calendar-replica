import React from 'react';
import { useStore, ymd } from '../store';
import type { EventInstance } from '../types';

const HOUR_H = 48; // px per hour
const LABEL_W = 56;

const startOf = (e: EventInstance): Date =>
  e.all_day ? new Date(e.occurrence_date + 'T00:00:00') : new Date(e.start);
const endOf = (e: EventInstance): Date =>
  e.all_day ? new Date(e.occurrence_date + 'T23:59:59') : new Date(e.end);
const fmtTime = (d: Date): string =>
  d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: false });

/* ---------------- Month ---------------- */

const MonthView: React.FC = () => {
  const { cursor, events, openCreate, openEdit } = useStore();
  const todayKey = ymd(new Date());

  const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
  const shift = (first.getDay() + 6) % 7;
  const gridStart = new Date(first);
  gridStart.setDate(first.getDate() - shift);

  const byDay = new Map<string, EventInstance[]>();
  for (const e of events) {
    const list = byDay.get(e.occurrence_date) ?? [];
    list.push(e);
    byDay.set(e.occurrence_date, list);
  }

  const cells = Array.from({ length: 42 }, (_, i) => {
    const d = new Date(gridStart);
    d.setDate(gridStart.getDate() + i);
    return d;
  });

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)' }}>
        {['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'].map((d) => (
          <div key={d} style={{ textAlign: 'center', fontSize: 11, color: '#70757a', padding: '4px 0', borderBottom: '1px solid #dadce0' }}>
            {d}
          </div>
        ))}
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', flex: 1, gridAutoRows: '1fr' }}>
        {cells.map((d, i) => {
          const key = ymd(d);
          const inMonth = d.getMonth() === cursor.getMonth();
          const isToday = key === todayKey;
          const dayEvents = byDay.get(key) ?? [];
          return (
            <div
              key={i}
              onClick={() => openCreate(new Date(d.getFullYear(), d.getMonth(), d.getDate(), 9, 0))}
              style={{
                borderRight: '1px solid #dadce0', borderBottom: '1px solid #dadce0',
                padding: 2, cursor: 'pointer', overflow: 'hidden',
                background: isToday ? '#e8f0fe' : '#fff',
              }}
            >
              <div style={{
                fontSize: 11, width: 20, height: 20, lineHeight: '20px', textAlign: 'center',
                borderRadius: '50%', margin: '0 auto 2px',
                background: isToday ? '#1a73e8' : 'transparent', color: isToday ? '#fff' : '#3c4043',
                opacity: inMonth ? 1 : 0.4, fontWeight: isToday ? 700 : 400,
              }}>
                {d.getDate()}
              </div>
              {dayEvents.slice(0, 4).map((e, j) => (
                <div
                  key={j}
                  onClick={(ev) => { ev.stopPropagation(); openEdit(e.event_id, e.occurrence_date); }}
                  style={{
                    fontSize: 10, marginBottom: 1, padding: '1px 3px', borderRadius: 3,
                    background: e.all_day ? e.color : '#fff',
                    color: e.all_day ? '#fff' : '#3c4043',
                    borderLeft: e.all_day ? 'none' : `3px solid ${e.color}`,
                    textDecoration: e.cancelled ? 'line-through' : 'none',
                    whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
                  }}
                >
                  {e.all_day ? e.title : `${fmtTime(startOf(e))} ${e.title}`}
                </div>
              ))}
              {dayEvents.length > 4 && (
                <div style={{ fontSize: 10, color: '#5f6368' }}>+{dayEvents.length - 4} more</div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

/* ------------- Week / Day time grid ------------- */

interface Slot { event: EventInstance; col: number; cols: number; top: number; height: number }

const layoutDay = (dayEvents: EventInstance[]): Slot[] => {
  const timed = dayEvents
    .filter((e) => !e.all_day)
    .sort((a, b) => startOf(a).getTime() - startOf(b).getTime());
  const slots: Slot[] = [];
  let cluster: EventInstance[] = [];
  let clusterEnd = 0;

  const flush = () => {
    if (!cluster.length) return;
    // Greedy column assignment within the cluster
    const colEnds: number[] = [];
    const assigned = cluster.map((e) => {
      let col = colEnds.findIndex((end) => end <= startOf(e).getTime());
      if (col === -1) { col = colEnds.length; }
      colEnds[col] = endOf(e).getTime();
      return { e, col };
    });
    const cols = colEnds.length;
    for (const { e, col } of assigned) {
      const s = startOf(e);
      const en = endOf(e);
      const top = ((s.getHours() * 60 + s.getMinutes()) / 60) * HOUR_H;
      const raw = ((en.getTime() - s.getTime()) / 3600000) * HOUR_H;
      slots.push({ event: e, col, cols, top, height: Math.max(raw, 22) });
    }
    cluster = [];
    clusterEnd = 0;
  };

  for (const e of timed) {
    if (cluster.length && startOf(e).getTime() >= clusterEnd) flush();
    cluster.push(e);
    clusterEnd = Math.max(clusterEnd, endOf(e).getTime());
  }
  flush();
  return slots;
};

const TimeGridView: React.FC<{ days: Date[] }> = ({ days }) => {
  const { events, openCreate, openEdit } = useStore();
  const now = new Date();
  const hours = Array.from({ length: 24 }, (_, h) => h);

  const eventsFor = (d: Date) => {
    const key = ymd(d);
    return events.filter((e) => e.occurrence_date === key);
  };

  return (
    <div style={{ flex: 1, overflowY: 'auto', minWidth: 0 }}>
      {/* Header row */}
      <div style={{ display: 'grid', gridTemplateColumns: `${LABEL_W}px repeat(${days.length}, 1fr)`, position: 'sticky', top: 0, background: '#fff', zIndex: 2, borderBottom: '1px solid #dadce0' }}>
        <div />
        {days.map((d) => {
          const isToday = ymd(d) === ymd(now);
          return (
            <div key={ymd(d)} style={{ textAlign: 'center', padding: '4px 0' }}>
              <div style={{ fontSize: 10, color: '#70757a', textTransform: 'uppercase' }}>
                {d.toLocaleDateString('en-US', { weekday: 'short' })}
              </div>
              <div style={{
                fontSize: 22, borderRadius: '50%', width: 36, height: 36, lineHeight: '36px', margin: '0 auto',
                background: isToday ? '#1a73e8' : 'transparent', color: isToday ? '#fff' : '#3c4043',
              }}>
                {d.getDate()}
              </div>
            </div>
          );
        })}
      </div>
      {/* All-day strip */}
      <div style={{ display: 'grid', gridTemplateColumns: `${LABEL_W}px repeat(${days.length}, 1fr)`, borderBottom: '1px solid #dadce0' }}>
        <div style={{ fontSize: 10, color: '#70757a', padding: 2 }}>All day</div>
        {days.map((d) => (
          <div key={`ad${ymd(d)}`} style={{ padding: 1, minHeight: 20 }}>
            {eventsFor(d).filter((e) => e.all_day).map((e, j) => (
              <div
                key={j}
                onClick={() => openEdit(e.event_id, e.occurrence_date)}
                style={{
                  fontSize: 10, padding: '1px 4px', borderRadius: 3, marginBottom: 1,
                  background: e.color, color: '#fff', cursor: 'pointer',
                  textDecoration: e.cancelled ? 'line-through' : 'none',
                  whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
                }}
              >
                {e.title}
              </div>
            ))}
          </div>
        ))}
      </div>
      {/* Hour grid */}
      <div style={{ display: 'grid', gridTemplateColumns: `${LABEL_W}px repeat(${days.length}, 1fr)`, position: 'relative' }}>
        <div>
          {hours.map((h) => (
            <div key={h} style={{ height: HOUR_H, fontSize: 10, color: '#70757a', textAlign: 'right', paddingRight: 4 }}>
              {h === 0 ? '' : `${String(h).padStart(2, '0')}:00`}
            </div>
          ))}
        </div>
        {days.map((d) => {
          const slots = layoutDay(eventsFor(d));
          const isToday = ymd(d) === ymd(now);
          return (
            <div
              key={ymd(d)}
              onClick={() => openCreate(new Date(d.getFullYear(), d.getMonth(), d.getDate(), 9, 0))}
              style={{ position: 'relative', borderLeft: '1px solid #dadce0' }}
            >
              {hours.map((h) => (
                <div key={h} style={{ height: HOUR_H, borderBottom: '1px solid #e8eaed' }} />
              ))}
              {isToday && (
                <div style={{
                  position: 'absolute', left: 0, right: 0,
                  top: ((now.getHours() * 60 + now.getMinutes()) / 60) * HOUR_H,
                  borderTop: '2px solid #d93025', zIndex: 1,
                }} />
              )}
              {slots.map((s, j) => (
                <div
                  key={j}
                  onClick={(ev) => { ev.stopPropagation(); openEdit(s.event.event_id, s.event.occurrence_date); }}
                  style={{
                    position: 'absolute', cursor: 'pointer', zIndex: 2,
                    top: s.top, height: s.height - 2,
                    left: `${(s.col / s.cols) * 100}%`,
                    width: `calc(${(1 / s.cols) * 100}% - 3px)`,
                    background: s.event.color, color: '#fff', borderRadius: 4,
                    fontSize: 10, padding: '2px 4px', overflow: 'hidden',
                    whiteSpace: 'nowrap', textOverflow: 'ellipsis',
                    textDecoration: s.event.cancelled ? 'line-through' : 'none',
                  }}
                  title={`${s.event.title} (${fmtTime(startOf(s.event))}–${fmtTime(endOf(s.event))})`}
                >
                  {s.event.title}
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
};

/* ---------------- Agenda ---------------- */

const AgendaView: React.FC = () => {
  const { events, openEdit } = useStore();
  const byDay = new Map<string, EventInstance[]>();
  for (const e of events) {
    const list = byDay.get(e.occurrence_date) ?? [];
    list.push(e);
    byDay.set(e.occurrence_date, list);
  }
  const days = [...byDay.keys()].sort();

  if (!days.length) {
    return <div style={{ flex: 1, padding: '2rem', color: '#70757a', textAlign: 'center' }}>No events for the next 4 days</div>;
  }

  return (
    <div style={{ flex: 1, overflowY: 'auto', padding: '8px 16px', minWidth: 0 }}>
      {days.map((day) => {
        const [y, m, d] = day.split('-').map(Number);
        const dt = new Date(y, m - 1, d);
        return (
          <div key={day} style={{ borderBottom: '1px solid #dadce0', padding: '8px 0' }}>
            <div style={{ fontWeight: 600, color: '#3c4043', marginBottom: 4 }}>
              {dt.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}
            </div>
            {(byDay.get(day) ?? [])
              .sort((a, b) => startOf(a).getTime() - startOf(b).getTime())
              .map((e, j) => (
                <div
                  key={j}
                  onClick={() => openEdit(e.event_id, e.occurrence_date)}
                  style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '3px 0', cursor: 'pointer' }}
                >
                  <span style={{ width: 10, height: 10, borderRadius: '50%', background: e.color, flexShrink: 0 }} />
                  <span style={{ width: 90, fontSize: 12, color: '#70757a' }}>
                    {e.all_day ? 'All day' : `${fmtTime(startOf(e))} – ${fmtTime(endOf(e))}`}
                  </span>
                  <span style={{ fontSize: 13, color: '#3c4043', textDecoration: e.cancelled ? 'line-through' : 'none' }}>
                    {e.title}
                  </span>
                </div>
              ))}
          </div>
        );
      })}
    </div>
  );
};

/* ---------------- Switcher ---------------- */

export const CalendarView: React.FC = () => {
  const { viewMode, cursor } = useStore();

  if (viewMode === 'month') return <MonthView />;

  if (viewMode === 'day') {
    const d = new Date(cursor.getFullYear(), cursor.getMonth(), cursor.getDate());
    return <TimeGridView days={[d]} />;
  }

  if (viewMode === 'week') {
    const start = new Date(cursor.getFullYear(), cursor.getMonth(), cursor.getDate());
    start.setDate(cursor.getDate() - ((cursor.getDay() + 6) % 7));
    const days = Array.from({ length: 7 }, (_, i) => {
      const d = new Date(start);
      d.setDate(start.getDate() + i);
      return d;
    });
    return <TimeGridView days={days} />;
  }

  return <AgendaView />;
};
