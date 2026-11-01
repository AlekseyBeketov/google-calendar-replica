import React, { useEffect } from 'react';
import { useStore } from './store';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { CalendarView } from './components/CalendarView';
import { TasksPanel } from './components/TasksPanel';
import { EventModal } from './components/EventModal';

const App: React.FC = () => {
  const { init } = useStore();
  useEffect(() => {
    init(); // load calendars, events, tasks
  }, []);

  return (
    <div className="app" style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <Header />
      <div style={{ display: 'flex', flex: 1 }}>
        <Sidebar />
        <CalendarView />
        <TasksPanel />
      </div>
      <EventModal />
    </div>
  );
};

export default App;
