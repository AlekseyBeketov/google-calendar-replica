import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  Calendar, CalendarInput, EventInput, EventInstance,
  EventPatch, Scope, Task, TaskInput, TaskPatch,
} from './types';

export type ViewMode = 'month' | 'week' | 'day' | 'agenda';

export interface ModalState {
  open: boolean;
  eventId?: string;
  /** Occurrence day 'YYYY-MM-DD' when editing a specific instance of a series. */
  occurrenceDate?: string;
  /** Preselected slot start (Date) when creating from a grid click. */
  slot?: Date;
}

/** Local Date -> 'YYYY-MM-DD' */
export const ymd = (d: Date): string => {
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${d.getFullYear()}-${m}-${day}`;
};

/** Visible range [start, end) for the current view, as local Dates. */
export const viewRange = (view: ViewMode, cursor: Date): [Date, Date] => {
  const day = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
  if (view === 'month') {
    const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const shift = (first.getDay() + 6) % 7; // Monday-start
    const start = new Date(first);
    start.setDate(first.getDate() - shift);
    const end = new Date(start);
    end.setDate(start.getDate() + 42);
    return [day(start), day(end)];
  }
  if (view === 'week') {
    const start = day(cursor);
    start.setDate(cursor.getDate() - ((cursor.getDay() + 6) % 7));
    const end = new Date(start);
    end.setDate(start.getDate() + 7);
    return [start, end];
  }
  if (view === 'day') {
    const start = day(cursor);
    const end = new Date(start);
    end.setDate(start.getDate() + 1);
    return [start, end];
  }
  // agenda: 4 days
  const start = day(cursor);
  const end = new Date(start);
  end.setDate(start.getDate() + 4);
  return [start, end];
};

interface StoreState {
  calendars: Calendar[];
  events: EventInstance[];
  tasks: Task[];
  viewMode: ViewMode;
  cursor: Date;
  modal: ModalState;
  loaded: boolean;

  init: () => Promise<void>;
  reloadEvents: () => Promise<void>;
  setViewMode: (v: ViewMode) => void;
  goToday: () => void;
  goPrev: () => void;
  goNext: () => void;
  setCursor: (d: Date) => void;
  openCreate: (slot?: Date) => void;
  openEdit: (eventId: string, occurrenceDate?: string) => void;
  closeModal: () => void;

  createEvent: (input: EventInput) => Promise<void>;
  updateEvent: (id: string, patch: EventPatch, scope: Scope, occurrenceDate?: string) => Promise<void>;
  deleteEvent: (id: string, scope: Scope, occurrenceDate?: string) => Promise<void>;
  addCalendar: (input: CalendarInput) => Promise<void>;
  toggleCalendar: (id: string, visible: boolean) => Promise<void>;
  createTask: (input: TaskInput) => Promise<void>;
  updateTask: (id: string, patch: TaskPatch) => Promise<void>;
  setTaskCompleted: (id: string, completed: boolean) => Promise<void>;
  deleteTask: (id: string) => Promise<void>;
}

export const useStore = create<StoreState>((set, get) => ({
  calendars: [],
  events: [],
  tasks: [],
  viewMode: 'month',
  cursor: new Date(),
  modal: { open: false },
  loaded: false,

  init: async () => {
    const [cals, tks] = await Promise.all([
      invoke<Calendar[]>('get_calendars'),
      invoke<Task[]>('list_tasks', { only_active: false }),
    ]);
    set({ calendars: cals, tasks: tks });
    await get().reloadEvents();
    set({ loaded: true });
  },

  reloadEvents: async () => {
    const { viewMode, cursor } = get();
    const [s, e] = viewRange(viewMode, cursor);
    const events = await invoke<EventInstance[]>('list_events_range', {
      start: s.toISOString(),
      end: e.toISOString(),
    });
    set({ events });
  },

  setViewMode: (v) => {
    set({ viewMode: v });
    void get().reloadEvents();
  },
  setCursor: (d) => {
    set({ cursor: d });
    void get().reloadEvents();
  },
  goToday: () => get().setCursor(new Date()),
  goPrev: () => {
    const { viewMode, cursor, setCursor } = get();
    const d = new Date(cursor);
    if (viewMode === 'month') d.setMonth(d.getMonth() - 1);
    else if (viewMode === 'week') d.setDate(d.getDate() - 7);
    else if (viewMode === 'day') d.setDate(d.getDate() - 1);
    else d.setDate(d.getDate() - 4);
    setCursor(d);
  },
  goNext: () => {
    const { viewMode, cursor, setCursor } = get();
    const d = new Date(cursor);
    if (viewMode === 'month') d.setMonth(d.getMonth() + 1);
    else if (viewMode === 'week') d.setDate(d.getDate() + 7);
    else if (viewMode === 'day') d.setDate(d.getDate() + 1);
    else d.setDate(d.getDate() + 4);
    setCursor(d);
  },

  openCreate: (slot) => set({ modal: { open: true, slot } }),
  openEdit: (eventId, occurrenceDate) => set({ modal: { open: true, eventId, occurrenceDate } }),
  closeModal: () => set({ modal: { open: false } }),

  createEvent: async (input) => {
    await invoke('create_event', { input });
    await get().reloadEvents();
  },
  updateEvent: async (id, patch, scope, occurrenceDate) => {
    await invoke('update_event', { id, patch, scope, occurrence_date: occurrenceDate ?? null });
    await get().reloadEvents();
  },
  deleteEvent: async (id, scope, occurrenceDate) => {
    await invoke('delete_event', { id, scope, occurrence_date: occurrenceDate ?? null });
    await get().reloadEvents();
  },

  addCalendar: async (input) => {
    await invoke('add_calendar', { input });
    const cals = await invoke<Calendar[]>('get_calendars');
    set({ calendars: cals });
  },
  toggleCalendar: async (id, visible) => {
    await invoke('set_calendar_visible', { id, visible });
    set({ calendars: get().calendars.map((c) => (c.id === id ? { ...c, visible } : c)) });
    await get().reloadEvents();
  },

  createTask: async (input) => {
    await invoke('create_task', { input });
    const tks = await invoke<Task[]>('list_tasks', { only_active: false });
    set({ tasks: tks });
  },
  updateTask: async (id, patch) => {
    await invoke('update_task', { id, patch });
    const tks = await invoke<Task[]>('list_tasks', { only_active: false });
    set({ tasks: tks });
  },
  setTaskCompleted: async (id, completed) => {
    await invoke('set_task_completed', { id, completed });
    const tks = await invoke<Task[]>('list_tasks', { only_active: false });
    set({ tasks: tks });
  },
  deleteTask: async (id) => {
    await invoke('delete_task', { id });
    const tks = await invoke<Task[]>('list_tasks', { only_active: false });
    set({ tasks: tks });
  },
}));
