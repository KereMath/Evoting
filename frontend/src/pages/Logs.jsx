import { useState, useEffect } from 'react'
import Navbar from '../components/Navbar'
import { getEvents } from '../services/api'

function Logs({ onLogout }) {
  const [events, setEvents] = useState([])
  const [loading, setLoading] = useState(true)
  const [autoRefresh, setAutoRefresh] = useState(true)

  useEffect(() => {
    loadEvents()

    if (autoRefresh) {
      const interval = setInterval(loadEvents, 2000) // Refresh every 2 seconds
      return () => clearInterval(interval)
    }
  }, [autoRefresh])

  const loadEvents = async () => {
    try {
      const response = await getEvents({ limit: 100 })
      setEvents(response.data)
      setLoading(false)
    } catch (error) {
      console.error('Failed to load events:', error)
      setLoading(false)
    }
  }

  const formatTime = (timestamp) => {
    return new Date(timestamp).toLocaleString('tr-TR', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }

  const getEventColor = (eventType) => {
    if (eventType.includes('created') || eventType.includes('registered')) return '#667eea'
    if (eventType.includes('deleted') || eventType.includes('removed')) return '#e74c3c'
    if (eventType.includes('voted')) return '#27ae60'
    return '#3498db'
  }

  if (loading) {
    return (
      <div>
        <Navbar onLogout={onLogout} />
        <div className="container">
          <p>Loading...</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <Navbar onLogout={onLogout} />
      <div className="container">
        <div className="card">
          <div className="card-header">
            <h1 className="card-title">System Logs & Data Flow</h1>
            <div style={{display: 'flex', alignItems: 'center', gap: '10px'}}>
              <label style={{display: 'flex', alignItems: 'center', gap: '5px', cursor: 'pointer'}}>
                <input
                  type="checkbox"
                  checked={autoRefresh}
                  onChange={(e) => setAutoRefresh(e.target.checked)}
                />
                <span>Auto Refresh</span>
              </label>
              <button onClick={loadEvents} className="btn btn-primary">
                Refresh Now
              </button>
            </div>
          </div>

          <div className="card-body">
            {events.length === 0 ? (
              <p style={{textAlign: 'center', color: '#7f8c8d', padding: '40px'}}>
                No events logged yet
              </p>
            ) : (
              <div>
                {events.map(event => (
                  <div
                    key={event.id}
                    className="log-entry"
                    style={{borderLeftColor: getEventColor(event.event_type)}}
                  >
                    <div className="log-time">
                      {formatTime(event.created_at)}
                    </div>
                    <div className="log-event">
                      {event.event_type}
                      {event.entity_type && (
                        <span style={{
                          marginLeft: '10px',
                          padding: '2px 8px',
                          background: '#f0f0f0',
                          borderRadius: '4px',
                          fontSize: '12px',
                          fontWeight: 'normal'
                        }}>
                          {event.entity_type}
                        </span>
                      )}
                    </div>
                    {event.entity_id && (
                      <div style={{fontSize: '12px', color: '#7f8c8d', marginTop: '5px'}}>
                        Entity ID: {event.entity_id}
                      </div>
                    )}
                    {event.data && (
                      <div className="log-data">
                        <strong>Data Flow:</strong>
                        <pre style={{
                          background: '#f8f9fa',
                          padding: '10px',
                          borderRadius: '4px',
                          marginTop: '5px',
                          overflow: 'auto'
                        }}>
                          {JSON.stringify(event.data, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Event Statistics</h2>
          </div>
          <div className="card-body">
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-label">Total Events</div>
                <div className="stat-value">{events.length}</div>
              </div>

              <div className="stat-card" style={{borderLeftColor: '#667eea'}}>
                <div className="stat-label">Creation Events</div>
                <div className="stat-value">
                  {events.filter(e => e.event_type.includes('created') || e.event_type.includes('registered')).length}
                </div>
              </div>

              <div className="stat-card" style={{borderLeftColor: '#27ae60'}}>
                <div className="stat-label">Vote Events</div>
                <div className="stat-value">
                  {events.filter(e => e.event_type.includes('voted')).length}
                </div>
              </div>

              <div className="stat-card" style={{borderLeftColor: '#e74c3c'}}>
                <div className="stat-label">Deletion Events</div>
                <div className="stat-value">
                  {events.filter(e => e.event_type.includes('deleted') || e.event_type.includes('removed')).length}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Logs
