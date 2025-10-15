import { useState, useEffect } from 'react'
import Navbar from '../components/Navbar'
import { getNetworkTopology, getEvents, getElections } from '../services/api'

function Monitoring({ onLogout }) {
  const [topology, setTopology] = useState(null)
  const [recentFlows, setRecentFlows] = useState([])
  const [elections, setElections] = useState([])
  const [selectedElection, setSelectedElection] = useState('all')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadMonitoringData()
    const interval = setInterval(loadMonitoringData, 3000) // Refresh every 3s
    return () => clearInterval(interval)
  }, [])

  const loadMonitoringData = async () => {
    try {
      const [topologyRes, eventsRes, electionsRes] = await Promise.all([
        getNetworkTopology(),
        getEvents({ limit: 50 }),
        getElections()
      ])

      setTopology(topologyRes.data)
      setRecentFlows(eventsRes.data)
      setElections(electionsRes.data.filter(e => e.phase === 4))
      setLoading(false)
    } catch (error) {
      console.error('Failed to load monitoring data:', error)
      setLoading(false)
    }
  }

  const getNodeColor = (nodeType) => {
    switch (nodeType) {
      case 'backend': return '#667eea'
      case 'database': return '#27ae60'
      case 'frontend': return '#f39c12'
      case 'ttp': return '#e67e22'
      case 'trustee': return '#e74c3c'
      case 'voter': return '#3498db'
      default: return '#95a5a6'
    }
  }

  const getNodeIcon = (nodeType) => {
    switch (nodeType) {
      case 'backend': return '🖥️'
      case 'database': return '💾'
      case 'frontend': return '🌐'
      case 'ttp': return '🛡️'
      case 'trustee': return '🔐'
      case 'voter': return '👤'
      default: return '📦'
    }
  }

  const getFilteredNodes = () => {
    if (!topology) return []
    if (selectedElection === 'all') return topology.nodes

    return topology.nodes.filter(node => {
      if (node.election_id === null) return true // System nodes
      return node.election_id === selectedElection
    })
  }

  const getNodesByType = (nodes) => {
    const system = nodes.filter(n => !n.election_id)
    const ttp = nodes.filter(n => n.node_type === 'ttp')
    const trustees = nodes.filter(n => n.node_type === 'trustee')
    const voters = nodes.filter(n => n.node_type === 'voter')

    return { system, ttp, trustees, voters }
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

  const filteredNodes = getFilteredNodes()
  const { system, ttp, trustees, voters } = getNodesByType(filteredNodes)

  return (
    <div>
      <Navbar onLogout={onLogout} />
      <div className="container">
        <h1 style={{marginBottom: '30px', color: '#2c3e50'}}>
          🎛️ System Monitoring & Orchestration
        </h1>

        {/* Statistics */}
        <div className="stats-grid">
          <div className="stat-card">
            <div className="stat-label">Total Containers</div>
            <div className="stat-value">{topology?.total_containers || 0}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#27ae60'}}>
            <div className="stat-label">Active Containers</div>
            <div className="stat-value">{topology?.active_containers || 0}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#e67e22'}}>
            <div className="stat-label">TTP Nodes</div>
            <div className="stat-value">{ttp.length}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#e74c3c'}}>
            <div className="stat-label">Trustees</div>
            <div className="stat-value">{trustees.length}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#3498db'}}>
            <div className="stat-label">Voters</div>
            <div className="stat-value">{voters.length}</div>
          </div>
        </div>

        {/* Election Filter */}
        {elections.length > 0 && (
          <div className="card" style={{marginBottom: '20px'}}>
            <div className="card-body" style={{padding: '15px'}}>
              <label style={{marginRight: '10px', fontWeight: 'bold'}}>Filter by Election:</label>
              <select
                value={selectedElection}
                onChange={(e) => setSelectedElection(e.target.value)}
                style={{
                  padding: '8px 12px',
                  border: '1px solid #ddd',
                  borderRadius: '4px',
                  fontSize: '14px',
                  minWidth: '200px'
                }}
              >
                <option value="all">All Elections</option>
                {elections.map(election => (
                  <option key={election.id} value={election.id}>
                    {election.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
        )}

        {/* Network Topology Visualization */}
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">🌐 Network Topology Map</h2>
          </div>
          <div className="card-body">
            {/* System Layer */}
            <div style={{marginBottom: '30px'}}>
              <h3 style={{fontSize: '16px', color: '#667eea', marginBottom: '15px'}}>System Infrastructure</h3>
              <div style={{
                display: 'flex',
                justifyContent: 'center',
                gap: '40px',
                padding: '20px',
                background: '#f8f9fa',
                borderRadius: '8px'
              }}>
                {system.map(node => (
                  <NodeCard key={node.id} node={node} getNodeColor={getNodeColor} getNodeIcon={getNodeIcon} />
                ))}
              </div>
            </div>

            {/* TTP Layer */}
            {ttp.length > 0 && (
              <div style={{marginBottom: '30px'}}>
                <h3 style={{fontSize: '16px', color: '#e67e22', marginBottom: '15px'}}>Trusted Third Party (TTP)</h3>
                <div style={{
                  display: 'flex',
                  justifyContent: 'center',
                  gap: '40px',
                  padding: '20px',
                  background: '#fff3e0',
                  borderRadius: '8px'
                }}>
                  {ttp.map(node => (
                    <NodeCard key={node.id} node={node} getNodeColor={getNodeColor} getNodeIcon={getNodeIcon} showDetails />
                  ))}
                </div>
              </div>
            )}

            {/* Trustees Layer */}
            {trustees.length > 0 && (
              <div style={{marginBottom: '30px'}}>
                <h3 style={{fontSize: '16px', color: '#e74c3c', marginBottom: '15px'}}>
                  Election Authorities (Trustees) - {trustees.length} active
                </h3>
                <div style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
                  gap: '15px',
                  padding: '20px',
                  background: '#ffebee',
                  borderRadius: '8px',
                  maxHeight: '400px',
                  overflowY: 'auto'
                }}>
                  {trustees.map(node => (
                    <NodeCard key={node.id} node={node} getNodeColor={getNodeColor} getNodeIcon={getNodeIcon} compact showDetails />
                  ))}
                </div>
              </div>
            )}

            {/* Voters Layer */}
            {voters.length > 0 && (
              <div>
                <h3 style={{fontSize: '16px', color: '#3498db', marginBottom: '15px'}}>
                  Voter Containers - {voters.length} active
                </h3>
                <div style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
                  gap: '15px',
                  padding: '20px',
                  background: '#e3f2fd',
                  borderRadius: '8px',
                  maxHeight: '400px',
                  overflowY: 'auto'
                }}>
                  {voters.map(node => (
                    <NodeCard key={node.id} node={node} getNodeColor={getNodeColor} getNodeIcon={getNodeIcon} compact showDetails />
                  ))}
                </div>
              </div>
            )}

            {filteredNodes.length === 3 && (
              <div style={{textAlign: 'center', padding: '40px', color: '#7f8c8d'}}>
                <p>No election containers running. Setup an election to see TTP, Trustees, and Voters here.</p>
              </div>
            )}
          </div>
        </div>

        {/* Real-time Event Logs */}
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">📋 System Event Logs</h2>
          </div>
          <div className="card-body">
            {recentFlows.length === 0 ? (
              <p style={{color: '#7f8c8d', textAlign: 'center', padding: '20px'}}>
                No recent events
              </p>
            ) : (
              <div style={{maxHeight: '400px', overflowY: 'auto'}}>
                {recentFlows.map(flow => (
                  <div
                    key={flow.id}
                    style={{
                      padding: '12px',
                      marginBottom: '8px',
                      background: '#f8f9fa',
                      borderRadius: '6px',
                      borderLeft: `4px solid ${getEventColor(flow.event_type)}`
                    }}
                  >
                    <div style={{display: 'flex', justifyContent: 'space-between', marginBottom: '5px'}}>
                      <div>
                        <strong style={{fontSize: '14px'}}>{flow.event_type.replace(/_/g, ' ').toUpperCase()}</strong>
                        {flow.entity_type && (
                          <span style={{
                            marginLeft: '8px',
                            padding: '2px 6px',
                            background: '#e0e0e0',
                            borderRadius: '3px',
                            fontSize: '11px'
                          }}>
                            {flow.entity_type}
                          </span>
                        )}
                      </div>
                      <div style={{fontSize: '11px', color: '#7f8c8d'}}>
                        {new Date(flow.created_at).toLocaleString('tr-TR')}
                      </div>
                    </div>
                    {flow.data && (
                      <div style={{
                        fontSize: '12px',
                        color: '#555',
                        fontFamily: 'monospace',
                        background: 'white',
                        padding: '6px',
                        borderRadius: '4px',
                        marginTop: '6px'
                      }}>
                        {Object.entries(flow.data).map(([key, value]) => (
                          <div key={key}>
                            <strong>{key}:</strong> {JSON.stringify(value)}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <style>{`
          @keyframes pulse {
            0%, 100% {
              box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            }
            50% {
              box-shadow: 0 8px 16px rgba(102, 126, 234, 0.4);
            }
          }
        `}</style>
      </div>
    </div>
  )
}

function NodeCard({ node, getNodeColor, getNodeIcon, compact = false, showDetails = false }) {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: '8px',
      padding: compact ? '12px' : '15px',
      background: 'white',
      borderRadius: '8px',
      border: '2px solid #e0e0e0',
      boxShadow: node.status === 'running' ? '0 2px 8px rgba(0,0,0,0.1)' : 'none',
      transition: 'all 0.3s'
    }}>
      <div style={{
        width: compact ? '60px' : '80px',
        height: compact ? '60px' : '80px',
        borderRadius: '50%',
        background: getNodeColor(node.node_type),
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'white',
        fontSize: compact ? '24px' : '32px',
        boxShadow: '0 4px 6px rgba(0,0,0,0.1)',
        border: '3px solid white',
        animation: node.status === 'running' ? 'pulse 2s infinite' : 'none'
      }}>
        {getNodeIcon(node.node_type)}
      </div>

      <div style={{
        textAlign: 'center',
        fontSize: compact ? '12px' : '14px',
        fontWeight: 'bold',
        color: '#2c3e50'
      }}>
        {node.name}
      </div>

      {node.port && (
        <div style={{
          padding: '4px 8px',
          background: '#ecf0f1',
          borderRadius: '12px',
          fontSize: '11px',
          fontFamily: 'monospace',
          color: '#555'
        }}>
          Port: {node.port}
        </div>
      )}

      {showDetails && node.metadata && (
        <div style={{
          fontSize: '10px',
          color: '#7f8c8d',
          textAlign: 'center',
          marginTop: '4px'
        }}>
          {node.metadata}
        </div>
      )}

      <div style={{
        padding: '3px 10px',
        background: node.status === 'running' ? '#27ae60' : '#95a5a6',
        color: 'white',
        borderRadius: '10px',
        fontSize: '10px',
        fontWeight: 'bold'
      }}>
        {node.status}
      </div>
    </div>
  )
}

function getEventColor(eventType) {
  if (eventType.includes('container')) return '#3498db'
  if (eventType.includes('trustee')) return '#e74c3c'
  if (eventType.includes('voter')) return '#3498db'
  if (eventType.includes('ttp')) return '#e67e22'
  return '#667eea'
}

export default Monitoring
