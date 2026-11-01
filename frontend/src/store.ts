import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Calendar, EventInstance, Task } from './types';

interface StoreState {
  calendars: Calendar[];
  events: EventInstance[];
  tasks: Task[];
  viewMode: 'month' | 'week' | 'day' | 'agenda';
  cursor: Date;
  init: () => void;
  // ... other actions like loadEvents, addCalendar, etc.
}

export const useStore = create<StoreState>((set, get) => ({
  calendars: [],
  events: [],
  tasks: [],
  viewMode: 'month',
  cursor: new Date(),
  init: async () => {
    const [cals, evs, tks] = await Promise.all([
      invoke<Calendar[]>('get_calendars'),
      // placeholder range: today +/- 7 days
      invoke<EventInstance[]>('list_events_range', {
        start: new Date().toISOString(),
        end: new Date(Date.now() + 14 * 24 * 60 * 60 * 1000).toISOString(),
      }),
      invoke<Task[]>('list_tasks', { only_active: true }),
    ]);
    set({ calendars: cals, events: evs, tasks: tks });
  },
  // additional actions would be added here
}));
