import React, { useState } from 'react';
import { useStore } from '../store';

export const TasksPanel: React.FC = () => {
  const { tasks, createTask, setTaskCompleted, deleteTask } = useStore();
  const [title, setTitle] = useState('');
  const [due, setDue] = useState('');

  const submit = async () => {
    if (!title.trim()) return;
    await createTask({ title: title.trim(), due_date: due || undefined });
    setTitle('');
    setDue('');
  };

  const active = tasks.filter((t) => t.status === 'needs_action');
  const completed = tasks.filter((t) => t.status === 'completed');

  const row = (t: (typeof tasks)[number]) => (
    <li key={t.id} style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '3px 0' }}>
      <input
        type="checkbox"
        checked={t.status === 'completed'}
        onChange={() => void setTaskCompleted(t.id, t.status === 'needs_action')}
      />
      <span style={{
        fontSize: 13, flex: 1,
        textDecoration: t.status === 'completed' ? 'line-through' : 'none',
        color: t.overdue ? '#d93025' : '#3c4043',
      }}>
        {t.title}
        {t.due_date && (
          <span style={{ fontSize: 11, color: t.overdue ? '#d93025' : '#70757a', marginLeft: 4 }}>
            {t.due_date}
          </span>
        )}
      </span>
      <button
        onClick={() => void deleteTask(t.id)}
        title="Delete"
        style={{ border: 'none', background: 'none', cursor: 'pointer', color: '#70757a' }}
      >
        ×
      </button>
    </li>
  );

  return (
    <aside style={{ width: 230, borderLeft: '1px solid #dadce0', padding: 8, flexShrink: 0 }}>
      <h3 style={{ fontSize: 14, color: '#3c4043', margin: '0 0 6px' }}>My Tasks</h3>
      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter') void submit(); }}
        placeholder="Add a task…"
        style={{ width: '100%', padding: '4px 6px', fontSize: 13, marginBottom: 4 }}
      />
      <input
        type="date"
        value={due}
        onChange={(e) => setDue(e.target.value)}
        title="Due date (optional)"
        style={{ width: '100%', fontSize: 12, marginBottom: 6, color: due ? '#3c4043' : '#70757a' }}
      />
      <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
        {active.map(row)}
        {completed.length > 0 && active.length > 0 && <hr style={{ border: 'none', borderTop: '1px solid #e8eaed' }} />}
        {completed.map(row)}
      </ul>
      {!tasks.length && <div style={{ fontSize: 12, color: '#70757a' }}>No tasks yet</div>}
    </aside>
  );
};
