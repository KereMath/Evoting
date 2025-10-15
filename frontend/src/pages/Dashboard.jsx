import { useState, useEffect } from 'react'
import Navbar from '../components/Navbar'
import { getElections, getVoters, getTrustees, getEvents } from '../services/api'

function Dashboard({ onLogout }) {
  const [stats, setStats] = useState({
    totalElections: 0,
    activeElections: 0,
    totalVoters: 0,
    totalTrustees: 0
  })
  const [recentEvents, setRecentEvents] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadDashboardData()
    // Auto-refresh every 5 seconds
    const interval = setInterval(loadDashboardData, 5000)
    return () => clearInterval(interval)
  }, [])

  const loadDashboardData = async () => {
    try {
      const [electionsRes, votersRes, trusteesRes, eventsRes] = await Promise.all([
        getElections(),
        getVoters(),
        getTrustees(),
        getEvents({ limit: 10 })
      ])

      const elections = electionsRes.data
      setStats({
        totalElections: elections.length,
        activeElections: elections.filter(e => e.status === 'active' || e.status === 'voting').length,
        totalVoters: votersRes.data.length,
        totalTrustees: trusteesRes.data.length
      })

      setRecentEvents(eventsRes.data)
      setLoading(false)
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
      setLoading(false)
    }
  }

  const formatEventTime = (timestamp) => {
    return new Date(timestamp).toLocaleString('tr-TR')
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
        <h1 style={{marginBottom: '30px', color: '#2c3e50'}}>Dashboard</h1>

        <div className="stats-grid">
          <div className="stat-card">
            <div className="stat-label">Total Elections</div>
            <div className="stat-value">{stats.totalElections}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#27ae60'}}>
            <div className="stat-label">Active Elections</div>
            <div className="stat-value">{stats.activeElections}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#f39c12'}}>
            <div className="stat-label">Total Voters</div>
            <div className="stat-value">{stats.totalVoters}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#e74c3c'}}>
            <div className="stat-label">Total Trustees</div>
            <div className="stat-value">{stats.totalTrustees}</div>
          </div>
        </div>

        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Recent System Events</h2>
          </div>
          <div className="card-body">
            {recentEvents.length === 0 ? (
              <p style={{color: '#7f8c8d'}}>No events yet</p>
            ) : (
              recentEvents.map(event => (
                <div key={event.id} className="log-entry">
                  <div className="log-time">{formatEventTime(event.created_at)}</div>
                  <div className="log-event">{event.event_type}</div>
                  {event.data && (
                    <div className="log-data">{JSON.stringify(event.data, null, 2)}</div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default Dashboard
