import { useState } from 'react';
import './index.css';

function App() {
  const [connected, setConnected] = useState(false);
  const [relayEnabled, setRelayEnabled] = useState(true);

  return (
    <div className="glass-panel">
      <div style={{ textAlign: 'center' }}>
        <h1 style={{ margin: 0, fontSize: '1.5rem', fontWeight: 700, letterSpacing: '-0.5px' }}>NexusVPN</h1>
        <p style={{ margin: '4px 0 0 0', color: '#9ca3af', fontSize: '0.875rem' }}>Next-Gen P2P Network</p>
      </div>

      <div className={`status-ring ${connected ? 'connected' : 'disconnected'}`}>
        <div className="status-inner">
          <p className="status-text">{connected ? 'SECURED' : 'UNPROTECTED'}</p>
          <p className="status-sub">{connected ? 'Seattle, WA' : 'Not Connected'}</p>
        </div>
      </div>

      <button 
        className="toggle-btn"
        style={{ 
          background: connected ? 'rgba(239, 68, 68, 0.1)' : 'rgba(59, 130, 246, 0.1)',
          borderColor: connected ? 'rgba(239, 68, 68, 0.3)' : 'rgba(59, 130, 246, 0.3)',
          color: connected ? '#ef4444' : '#60a5fa'
        }}
        onClick={() => setConnected(!connected)}
      >
        {connected ? 'DISCONNECT' : 'QUICK CONNECT'}
      </button>

      <div className="stats-grid">
        <div className="stat-box">
          <div className="stat-val">{connected ? '14' : '0'}</div>
          <div className="stat-label">Ping (ms)</div>
        </div>
        <div className="stat-box">
          <div className="stat-val">{relayEnabled ? 'ON' : 'OFF'}</div>
          <div className="stat-label">Relay Mode</div>
        </div>
      </div>

      <button 
        className="toggle-btn" 
        onClick={() => setRelayEnabled(!relayEnabled)}
        style={{ fontSize: '0.875rem', opacity: 0.8 }}
      >
        {relayEnabled ? 'Pause Background Relay' : 'Enable Background Relay'}
      </button>
    </div>
  );
}

export default App;
