export type Scope = 'this' | 'this_and_following' | 'all';

export type EventStatus = 'confirmed' | 'tentative' | 'cancelled';
export type TaskStatus = 'needs_action' | 'completed';

export interface Calendar {
  id: string;
  name: string;
  color: string;
  is_primary: boolean;
  visible: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface CalendarInput {
  name: string;
  color: string;
}

export interface Event {
  id: string;
  calendar_id: string;
  series_id?: string;
  title: string;
  all_day: boolean;
  start_utc?: string;
  end_utc?: string;
  tz_id?: string;
  start_date?: string;
  end_date?: string;
  rrule?: string;
  description: string;
  location: string;
  reminder_minutes?: number;
  status: EventStatus;
  created_at: string;
  updated_at: string;
}

export interface EventInput {
  calendar_id: string;
  title: string;
  all_day?: boolean;
  start_utc?: string;
  end_utc?: string;
  tz_id?: string;
  start_date?: string;
  end_date?: string;
  description?: string;
  location?: string;
  reminder_minutes?: number;
  status?: EventStatus;
  rrule?: string;
}

export interface EventPatch {
  calendar_id?: string;
  title?: string;
  all_day?: boolean;
  start_utc?: string;
  end_utc?: string;
  tz_id?: string;
  start_date?: string;
  end_date?: string;
  rrule?: string;
  description?: string;
  location?: string;
  reminder_minutes?: number;
  status?: EventStatus;
}

export interface EventInstance {
  event_id: string;
  series_id?: string;
  occurrence_date: string;
  start: string;
  end: string;
  all_day: boolean;
  title: string;
  calendar_id: string;
  color: string;
  is_exception: boolean;
  cancelled: boolean;
}

export interface Task {
  id: string;
  title: string;
  notes: string;
  due_date?: string;
  status: TaskStatus;
  position: number;
  created_at: string;
  completed_at?: string;
  overdue: boolean;
}

export interface TaskInput {
  title: string;
  notes?: string;
  due_date?: string;
}

export interface TaskPatch {
  title?: string;
  notes?: string;
  due_date?: string;
}

export interface CompleteInput {
  completed: boolean;
}
