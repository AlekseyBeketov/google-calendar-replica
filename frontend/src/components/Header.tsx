import React from 'react';

export const Header: React.FC = () => {
  return (
    <header style={{ padding: '0.5rem', backgroundColor: '#f5f5f5', borderBottom: '1px solid #ddd', display: 'flex', alignItems: 'center' }}>
      <button style={{ marginRight: '0.5rem' }}>Today</button>
      <button style={{ marginRight: '0.5rem' }}>{'<'}</button>
      <button style={{ marginRight: '0.5rem' }}>{'>'}</button>
      <span style={{ marginLeft: '1rem' }}>Date Range</span>
      <select style={{ marginLeft: '1rem' }} defaultValue="month">
        <option value="month">Month</option>
        <option value="week">Week</option>
        <option value="day">Day</option>
        <option value="agenda">Agenda</option>
      </select>
    </header>
  );
};
