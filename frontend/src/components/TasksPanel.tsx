import React, { useState } from 'react';
import { useStore } from '../store';
import type { Task, TaskInput } from '../types';
import { invoke } from '@tauri-apps/api/core';

export const TasksPanel: React.FC = () => {
  const { tasks, init } = useStore();
  const [title, setTitle] = useState('');

  const addTask = async () => {
    if (!title.trim()) return;
    const input: TaskInput = { title: title.trim() };
    await invoke('create_task', { input });
    setTitle('');
    init();
  };

  const toggleTask = async (task: Task) => {
    const completed = task.status === 'needs_action';
    await invoke('set_task_completed', { id: task.id, completed });
    init();
  };

  const deleteTask = async (task: Task) => {
    await invoke('delete_task', { id: task.id });
    init();
  };

  return (
    <aside style={{ width: '220px', borderLeft: '1px solid #ddd', padding: '0.5rem' }}>
      <h3>Tasks</h3>
      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter') addTask(); }}
        placeholder="Add a task…"
        style={{ width: '100%', marginBottom: '0.5rem' }}
      />
      <ul style={{ listStyle: 'none', padding: 0 }}>
        {tasks.map((t) => (
          <li key={t.id} style={{ display: 'flex', alignItems: 'center', marginBottom: '4px' }}>
            <input
              type="checkbox"
              checked={t.status === 'completed'}
              onChange={() => toggleTask(t)}
              style={{ marginRight: '0.5rem' }}
            />
            <span style={{
              textDecoration: t.status === 'completed' ? 'line-through' : 'none',
              color: t.overdue ? '#d93025' : 'inherit',
              flex: 1,
            }}>{t.title}</span>
            <button onClick={() => deleteTask(t)} title="Delete">×</button>
          </li>
        ))}
      </ul>
    </aside>
  );
};
