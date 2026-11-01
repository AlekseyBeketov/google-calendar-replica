import React from 'react';

export const Sidebar: React.FC = () => {
  return (
    <aside style={{ width: '200px', borderRight: '1px solid #ddd', padding: '0.5rem' }}>
      <h3>My Calendars</h3>
      {/* Placeholder list */}
      <ul style={{ listStyle: 'none', padding: 0 }}>
        <li><input type="checkbox" defaultChecked /> <span style={{ color: '#ff5555' }}>Work</span></li>
        <li><input type="checkbox" /> <span style={{ color: '#55aaff' }}>Personal</span></li>
      </ul>
      <hr />
      <h4>Mini Calendar</h4>
      {/* Simple mini calendar: show current month days */}
      <div>{new Date().toDateString()}</div>
    </aside>
  );
};
