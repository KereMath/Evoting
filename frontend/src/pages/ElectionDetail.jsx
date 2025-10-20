import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import Navbar from '../components/Navbar'
import { getElection, getVoters, createVoter, deleteVoter, getTrustees, createTrustee, uploadVotersCSV, setupElection, setupCrypto, getCryptoParameters, startKeygen, getKeygenStatus } from '../services/api'

function ElectionDetail({ onLogout }) {
  const { id } = useParams()
  const navigate = useNavigate()
  const [election, setElection] = useState(null)
  const [voters, setVoters] = useState([])
  const [trustees, setTrustees] = useState([])
  const [showVoterModal, setShowVoterModal] = useState(false)
  const [showTrusteeModal, setShowTrusteeModal] = useState(false)
  const [showCsvModal, setShowCsvModal] = useState(false)
  const [loading, setLoading] = useState(true)
  const [voterTcId, setVoterTcId] = useState('')
  const [trusteeName, setTrusteeName] = useState('')
  const [trusteeDockerType, setTrusteeDockerType] = useState('auto')
  const [trusteeIpAddress, setTrusteeIpAddress] = useState('')
  const [csvFile, setCsvFile] = useState(null)
  const [csvUploadResult, setCsvUploadResult] = useState(null)
  const [setupLoading, setSetupLoading] = useState(false)
  const [cryptoSetupLoading, setCryptoSetupLoading] = useState(false)
  const [cryptoParams, setCryptoParams] = useState(null)
  const [showCryptoModal, setShowCryptoModal] = useState(false)
  const [keygenLoading, setKeygenLoading] = useState(false)
  const [keygenStatus, setKeygenStatus] = useState(null)

  useEffect(() => {
    loadElectionData()
  }, [id])

  // Auto-poll DKG status when in key generation phase
  useEffect(() => {
    let pollInterval = null

    if (election && election.phase === 6 && election.status === 'key_generation') {
      // Start polling immediately
      const pollStatus = async () => {
        try {
          const statusRes = await getKeygenStatus(id)
          setKeygenStatus(statusRes.data)

          // If completed, stop polling and reload election data
          if (statusRes.data.status === 'completed') {
            if (pollInterval) clearInterval(pollInterval)
            loadElectionData()
          }
        } catch (error) {
          console.error('Failed to poll keygen status:', error)
        }
      }

      // Poll immediately
      pollStatus()

      // Then poll every 2 seconds
      pollInterval = setInterval(pollStatus, 2000)
    }

    // Cleanup
    return () => {
      if (pollInterval) clearInterval(pollInterval)
    }
  }, [election, id])

  const loadElectionData = async () => {
    try {
      const [electionRes, votersRes, trusteesRes] = await Promise.all([
        getElection(id),
        getVoters(id),
        getTrustees()
      ])

      setElection(electionRes.data)
      setVoters(votersRes.data)
      // Filter trustees for this election
      const electionTrustees = trusteesRes.data.filter(t => t.election_id === id)
      setTrustees(electionTrustees)

      // Load crypto parameters if election phase >= 5
      if (electionRes.data.phase >= 5) {
        try {
          const cryptoRes = await getCryptoParameters(id)
          setCryptoParams(cryptoRes.data)
        } catch (error) {
          console.error('Failed to load crypto parameters:', error)
        }
      }

      // Load DKG status if phase >= 6
      if (electionRes.data.phase >= 6) {
        try {
          const statusRes = await getKeygenStatus(id)
          setKeygenStatus(statusRes.data)
        } catch (error) {
          console.error('Failed to load keygen status:', error)
        }
      }

      setLoading(false)
    } catch (error) {
      console.error('Failed to load election data:', error)
      setLoading(false)
    }
  }

  const handleAddVoter = async (e) => {
    e.preventDefault()
    try {
      await createVoter({
        election_id: id,
        tc_id: voterTcId
      })
      setShowVoterModal(false)
      setVoterTcId('')
      loadElectionData()
    } catch (error) {
      console.error('Failed to add voter:', error)
      alert('Failed to add voter. TC ID might already be registered.')
    }
  }

  const handleDeleteVoter = async (voterId, tcId) => {
    if (window.confirm(`Remove voter with TC ID: ${tcId}?`)) {
      try {
        await deleteVoter(voterId)
        loadElectionData()
      } catch (error) {
        console.error('Failed to delete voter:', error)
        alert('Failed to delete voter')
      }
    }
  }

  const handleAddTrustee = async (e) => {
    e.preventDefault()
    try {
      await createTrustee({
        election_id: id,
        name: trusteeName,
        docker_type: trusteeDockerType,
        ip_address: trusteeDockerType === 'manual' ? trusteeIpAddress : null
      })
      setShowTrusteeModal(false)
      setTrusteeName('')
      setTrusteeDockerType('auto')
      setTrusteeIpAddress('')
      loadElectionData()
    } catch (error) {
      console.error('Failed to add trustee:', error)
      alert('Failed to add trustee')
    }
  }

  const handleSetup = async () => {
    if (!window.confirm('Are you sure you want to setup this election? This will create Docker containers for all voters and trustees.')) {
      return
    }

    setSetupLoading(true)
    try {
      const response = await setupElection(id)
      alert(response.data.message)
      loadElectionData()
    } catch (error) {
      console.error('Failed to setup election:', error)
      alert('Failed to setup election')
    } finally {
      setSetupLoading(false)
    }
  }

  const handleCryptoSetup = async () => {
    if (!window.confirm('Are you sure you want to setup cryptographic parameters? This will generate PBC parameters for the election.')) {
      return
    }

    setCryptoSetupLoading(true)
    try {
      const response = await setupCrypto(id, 256)
      alert(response.data.message)
      loadElectionData()
    } catch (error) {
      console.error('Failed to setup crypto:', error)
      alert('Failed to setup cryptographic parameters')
    } finally {
      setCryptoSetupLoading(false)
    }
  }

  const handleStartKeygen = async () => {
    if (!window.confirm('Are you sure you want to start Distributed Key Generation? All trustee containers will participate in the DKG protocol.')) {
      return
    }

    setKeygenLoading(true)
    try {
      const response = await startKeygen(id)
      alert(response.data.message)
      loadElectionData()

      // Poll for status
      const pollInterval = setInterval(async () => {
        try {
          const statusRes = await getKeygenStatus(id)
          setKeygenStatus(statusRes.data)

          if (statusRes.data.status === 'completed') {
            clearInterval(pollInterval)
            alert('Key generation completed successfully!')
            loadElectionData()
          }
        } catch (error) {
          console.error('Failed to poll keygen status:', error)
        }
      }, 3000)

      // Clear polling after 5 minutes
      setTimeout(() => clearInterval(pollInterval), 300000)
    } catch (error) {
      console.error('Failed to start keygen:', error)
      alert('Failed to start key generation')
    } finally {
      setKeygenLoading(false)
    }
  }

  const handleCsvUpload = async (e) => {
    e.preventDefault()
    if (!csvFile) {
      alert('Please select a CSV file')
      return
    }

    const formData = new FormData()
    formData.append('file', csvFile)

    try {
      const response = await uploadVotersCSV(formData)
      setCsvUploadResult(response.data)
      setCsvFile(null)
      loadElectionData()
    } catch (error) {
      console.error('Failed to upload CSV:', error)
      alert('Failed to upload CSV file')
    }
  }

  const getStatusBadge = (status) => {
    return <span className={`badge badge-${status}`}>{status}</span>
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

  if (!election) {
    return (
      <div>
        <Navbar onLogout={onLogout} />
        <div className="container">
          <p>Election not found</p>
        </div>
      </div>
    )
  }

  return (
    <div>
      <Navbar onLogout={onLogout} />
      <div className="container">
        <button
          onClick={() => navigate('/elections')}
          className="btn"
          style={{marginBottom: '20px', background: '#95a5a6', color: 'white'}}
        >
          ← Back to Elections
        </button>

        <div className="card">
          <div className="card-header">
            <h1 className="card-title">{election.name}</h1>
            {getStatusBadge(election.status)}
          </div>
          <div className="card-body">
            <p><strong>ID:</strong> {election.id}</p>
            {election.description && <p><strong>Description:</strong> {election.description}</p>}
            <p><strong>Threshold:</strong> {election.threshold} / {election.total_trustees}</p>
            <p><strong>Created:</strong> {new Date(election.created_at).toLocaleString('tr-TR')}</p>
          </div>
        </div>

        {/* Phase Flow */}
        <div className="card" style={{marginTop: '20px'}}>
          <div className="card-header">
            <h2 className="card-title">Election Phase</h2>
          </div>
          <div className="card-body">
            <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '20px'}}>
              {[1, 2, 3, 4, 5, 6].map((phase) => (
                <div key={phase} style={{flex: 1, display: 'flex', alignItems: 'center'}}>
                  <div style={{
                    width: '50px',
                    height: '50px',
                    borderRadius: '50%',
                    background: election.phase >= phase ? '#27ae60' : '#ecf0f1',
                    color: election.phase >= phase ? 'white' : '#95a5a6',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontWeight: 'bold',
                    fontSize: '20px',
                    boxShadow: election.phase === phase ? '0 0 0 4px rgba(39, 174, 96, 0.3)' : 'none'
                  }}>
                    {phase}
                  </div>
                  {phase !== 6 && <div style={{flex: 1, height: '4px', background: election.phase > phase ? '#27ae60' : '#ecf0f1', marginLeft: '10px'}} />}
                </div>
              ))}
            </div>
            <div style={{display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: '10px', marginTop: '10px'}}>
              <div>
                <strong>Phase 1:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Add Trustees</p>
              </div>
              <div>
                <strong>Phase 2:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Add Voters</p>
              </div>
              <div>
                <strong>Phase 3:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Ready for Setup</p>
              </div>
              <div>
                <strong>Phase 4:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Containers Ready</p>
              </div>
              <div>
                <strong>Phase 5:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Crypto Setup</p>
              </div>
              <div>
                <strong>Phase 6:</strong>
                <p style={{fontSize: '12px', color: '#7f8c8d', margin: '5px 0'}}>Key Generation</p>
              </div>
            </div>
            {election.phase === 3 && (
              <button
                onClick={handleSetup}
                disabled={setupLoading}
                className="btn btn-primary"
                style={{marginTop: '20px', width: '100%', padding: '12px', fontSize: '16px'}}
              >
                {setupLoading ? 'Setting up...' : '🚀 Setup Election (Create Containers)'}
              </button>
            )}
            {election.phase === 4 && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '12px', background: '#d4edda', border: '1px solid #28a745', borderRadius: '4px', marginBottom: '15px'}}>
                  <strong style={{color: '#155724'}}>✅ Election Setup Complete!</strong>
                  <p style={{fontSize: '12px', color: '#155724', margin: '5px 0'}}>
                    TTP Port: {election.ttp_port || 'N/A'} | Network: {election.docker_network || 'N/A'}
                  </p>
                </div>
                <button
                  onClick={handleCryptoSetup}
                  disabled={cryptoSetupLoading}
                  className="btn btn-primary"
                  style={{width: '100%', padding: '12px', fontSize: '16px', background: '#9b59b6'}}
                >
                  {cryptoSetupLoading ? 'Generating cryptographic parameters...' : '🔐 Setup Cryptographic Parameters'}
                </button>
              </div>
            )}
            {election.phase === 5 && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '12px', background: '#e8daef', border: '1px solid #9b59b6', borderRadius: '4px', marginBottom: '10px'}}>
                  <strong style={{color: '#6c3483'}}>🔐 Cryptographic Parameters Generated!</strong>
                  <p style={{fontSize: '12px', color: '#6c3483', margin: '5px 0'}}>
                    Election is ready for key generation phase
                  </p>
                </div>
                <div style={{display: 'flex', gap: '10px'}}>
                  {cryptoParams && (
                    <button
                      onClick={() => setShowCryptoModal(true)}
                      className="btn"
                      style={{flex: '1', padding: '12px', fontSize: '14px', background: '#8e44ad', color: 'white'}}
                    >
                      📋 View Cryptographic Parameters
                    </button>
                  )}
                  <button
                    onClick={handleStartKeygen}
                    disabled={keygenLoading}
                    className="btn btn-primary"
                    style={{flex: '2', padding: '12px', fontSize: '16px', background: '#e74c3c', color: 'white'}}
                  >
                    {keygenLoading ? 'Starting DKG...' : '🔑 Start Key Generation (DKG)'}
                  </button>
                </div>
              </div>
            )}
            {election.phase === 6 && election.status === 'key_generation' && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '15px', background: '#fff3cd', border: '1px solid #ffc107', borderRadius: '4px', marginBottom: '15px'}}>
                  <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px'}}>
                    <strong style={{color: '#856404', fontSize: '16px'}}>🔄 Distributed Key Generation In Progress...</strong>
                    {keygenStatus && (
                      <span style={{fontSize: '14px', color: '#856404'}}>
                        Step {keygenStatus.current_step}/7
                      </span>
                    )}
                  </div>

                  {/* Progress Bar */}
                  {keygenStatus && (
                    <div>
                      <div style={{
                        width: '100%',
                        height: '24px',
                        background: '#f8f9fa',
                        borderRadius: '12px',
                        overflow: 'hidden',
                        border: '1px solid #dee2e6',
                        marginBottom: '10px'
                      }}>
                        <div style={{
                          width: `${(keygenStatus.current_step / 7) * 100}%`,
                          height: '100%',
                          background: 'linear-gradient(90deg, #ffc107 0%, #ff9800 100%)',
                          transition: 'width 0.5s ease',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          color: 'white',
                          fontWeight: 'bold',
                          fontSize: '12px'
                        }}>
                          {Math.round((keygenStatus.current_step / 7) * 100)}%
                        </div>
                      </div>

                      {/* Step Description */}
                      <div style={{fontSize: '13px', color: '#6c757d', marginBottom: '12px'}}>
                        <strong>Current Step:</strong>{' '}
                        {keygenStatus.current_step === 0 && 'Initializing...'}
                        {keygenStatus.current_step === 1 && 'Step 1: Generating polynomials and broadcasting commitments'}
                        {keygenStatus.current_step === 2 && 'Step 2: Distributing secret shares to trustees'}
                        {keygenStatus.current_step === 3 && 'Step 3: Verifying received shares'}
                        {keygenStatus.current_step === 4 && 'Step 4: Broadcasting complaints (if any)'}
                        {keygenStatus.current_step === 5 && 'Step 5: Resolving complaints and determining qualified set'}
                        {keygenStatus.current_step === 6 && 'Step 6: Computing Master Verification Key (MVK)'}
                        {keygenStatus.current_step === 7 && 'Step 7: Computing signing keys and verification keys'}
                      </div>

                      {/* Trustee Status */}
                      <div style={{marginTop: '15px'}}>
                        <strong style={{fontSize: '13px', color: '#495057', display: 'block', marginBottom: '8px'}}>
                          Trustee Status ({keygenStatus.trustees_ready.filter(t => t.status === 'completed').length}/{keygenStatus.total_trustees} ready):
                        </strong>
                        <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '8px'}}>
                          {keygenStatus.trustees_ready.map((trustee) => (
                            <div key={trustee.trustee_id} style={{
                              padding: '8px 12px',
                              background: trustee.status === 'completed' ? '#d4edda' :
                                        trustee.status === 'in_progress' ? '#fff3cd' : '#f8d7da',
                              border: `1px solid ${trustee.status === 'completed' ? '#28a745' :
                                                  trustee.status === 'in_progress' ? '#ffc107' : '#dc3545'}`,
                              borderRadius: '4px',
                              fontSize: '12px'
                            }}>
                              <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between'}}>
                                <span style={{fontWeight: 'bold'}}>
                                  {trustee.status === 'completed' && '✅ '}
                                  {trustee.status === 'in_progress' && '⏳ '}
                                  {trustee.status === 'failed' && '❌ '}
                                  {trustee.name}
                                </span>
                                <span style={{fontSize: '11px', color: '#6c757d'}}>
                                  Step {trustee.current_step}
                                </span>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}

                  <p style={{fontSize: '12px', color: '#856404', margin: '12px 0 0 0'}}>
                    ℹ️ This process may take 1-2 minutes. Trustees are executing the Pedersen DKG protocol...
                  </p>
                </div>
              </div>
            )}
            {election.phase >= 6 && election.status !== 'key_generation' && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '12px', background: '#d4edda', border: '1px solid #28a745', borderRadius: '4px', marginBottom: '10px'}}>
                  <strong style={{color: '#155724'}}>✅ Key Generation Complete!</strong>
                  <p style={{fontSize: '12px', color: '#155724', margin: '5px 0'}}>
                    Master Verification Key (MVK) has been generated by trustees
                  </p>
                </div>
                {cryptoParams && (
                  <button
                    onClick={() => setShowCryptoModal(true)}
                    className="btn"
                    style={{width: '100%', padding: '12px', fontSize: '14px', background: '#8e44ad', color: 'white'}}
                  >
                    📋 View Cryptographic Parameters
                  </button>
                )}
              </div>
            )}
          </div>
        </div>

        {/* DKG Results - Public Keys (MVK and VKs) */}
        {election.phase >= 6 && keygenStatus && keygenStatus.mvk && (
          <div className="card" style={{marginTop: '20px'}}>
            <div className="card-header" style={{background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', color: 'white'}}>
              <h2 className="card-title">🔑 DKG Results - Public Keys</h2>
              <span style={{fontSize: '12px', opacity: 0.9}}>
                These keys are public and visible to all participants
              </span>
            </div>
            <div className="card-body">
              {/* Master Verification Key (MVK) */}
              <div style={{marginBottom: '20px', padding: '15px', background: '#f8f9fa', borderRadius: '8px', border: '2px solid #667eea'}}>
                <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px'}}>
                  <strong style={{color: '#667eea', fontSize: '16px'}}>
                    🔐 Master Verification Key (MVK)
                  </strong>
                  <span style={{fontSize: '11px', background: '#667eea', color: 'white', padding: '4px 8px', borderRadius: '4px'}}>
                    PUBLIC
                  </span>
                </div>
                <pre style={{
                  background: 'white',
                  padding: '12px',
                  borderRadius: '4px',
                  fontSize: '11px',
                  fontFamily: 'monospace',
                  overflow: 'auto',
                  maxHeight: '150px',
                  border: '1px solid #dee2e6',
                  margin: 0
                }}>
                  {keygenStatus.mvk ? JSON.stringify(keygenStatus.mvk, null, 2) : 'Not yet generated'}
                </pre>
                <p style={{fontSize: '12px', color: '#6c757d', margin: '8px 0 0 0'}}>
                  ℹ️ Used to verify threshold signatures from qualified trustees
                </p>
              </div>

              {/* Trustee Verification Keys (VKs) */}
              <div>
                <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px'}}>
                  <strong style={{color: '#495057', fontSize: '15px'}}>
                    📋 Trustee Verification Keys (VK_i)
                  </strong>
                  <span style={{fontSize: '11px', background: '#28a745', color: 'white', padding: '4px 8px', borderRadius: '4px'}}>
                    PUBLIC
                  </span>
                </div>

                {keygenStatus.trustees_ready && keygenStatus.trustees_ready.filter(t => t.status === 'completed' && t.verification_key).length > 0 ? (
                  <div style={{display: 'grid', gap: '12px'}}>
                    {keygenStatus.trustees_ready.filter(t => t.status === 'completed' && t.verification_key).map((trustee, idx) => (
                      <div key={trustee.trustee_id} style={{
                        padding: '12px',
                        background: '#fff',
                        border: '1px solid #dee2e6',
                        borderRadius: '6px'
                      }}>
                        <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px'}}>
                          <span style={{fontWeight: 'bold', color: '#495057'}}>
                            {trustee.name} <span style={{color: '#6c757d', fontSize: '12px'}}>(Index {trustee.trustee_index})</span>
                          </span>
                          <span style={{fontSize: '10px', background: '#e7f3ff', color: '#0066cc', padding: '3px 6px', borderRadius: '3px'}}>
                            VK_{idx + 1}
                          </span>
                        </div>
                        <pre style={{
                          background: '#f8f9fa',
                          padding: '8px',
                          borderRadius: '4px',
                          fontSize: '10px',
                          fontFamily: 'monospace',
                          overflow: 'auto',
                          maxHeight: '100px',
                          margin: 0
                        }}>
                          {trustee.verification_key || 'Not available'}
                        </pre>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p style={{color: '#6c757d', fontSize: '13px', padding: '12px', background: '#f8f9fa', borderRadius: '4px'}}>
                    No verification keys available yet
                  </p>
                )}

                <p style={{fontSize: '12px', color: '#6c757d', margin: '12px 0 0 0', padding: '10px', background: '#e7f3ff', borderRadius: '4px', border: '1px solid #bee5eb'}}>
                  ℹ️ <strong>Note:</strong> Private signing keys (SGK_i) are stored securely within each trustee's container and are never shared.
                </p>
              </div>
            </div>
          </div>
        )}

        <div className="stats-grid">
          <div className="stat-card">
            <div className="stat-label">Registered Voters</div>
            <div className="stat-value">{voters.length}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#e74c3c'}}>
            <div className="stat-label">Registered Trustees</div>
            <div className="stat-value">{trustees.length} / {election.total_trustees}</div>
          </div>

          <div className="stat-card" style={{borderLeftColor: '#27ae60'}}>
            <div className="stat-label">Votes Cast</div>
            <div className="stat-value">{voters.filter(v => v.status === 'voted').length}</div>
          </div>
        </div>

        {/* Trustees Section */}
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Trustees (Election Authorities)</h2>
            <button
              onClick={() => setShowTrusteeModal(true)}
              className="btn btn-primary"
              disabled={trustees.length >= election.total_trustees}
            >
              + Add Trustee
            </button>
          </div>
          <div className="card-body">
            {trustees.length === 0 ? (
              <p style={{color: '#7f8c8d'}}>No trustees registered yet</p>
            ) : (
              <table className="table">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Type</th>
                    <th>Connection</th>
                    <th>Status</th>
                    <th>Created At</th>
                  </tr>
                </thead>
                <tbody>
                  {trustees.map(trustee => (
                    <tr key={trustee.id}>
                      <td><strong>{trustee.name}</strong></td>
                      <td>
                        <span className={`badge badge-${trustee.docker_type === 'auto' ? 'info' : 'warning'}`}>
                          {trustee.docker_type === 'auto' ? 'Docker Auto' : 'Manual IP'}
                        </span>
                      </td>
                      <td style={{fontSize: '12px', fontFamily: 'monospace'}}>
                        {trustee.docker_type === 'auto'
                          ? (trustee.docker_port ? `Port: ${trustee.docker_port}` : 'Pending')
                          : (trustee.ip_address || 'N/A')}
                      </td>
                      <td>{getStatusBadge(trustee.status)}</td>
                      <td>{new Date(trustee.created_at).toLocaleString('tr-TR')}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>

        {/* Voters Section */}
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Voters</h2>
            <div style={{display: 'flex', gap: '10px'}}>
              <button
                onClick={() => setShowCsvModal(true)}
                className="btn"
                style={{background: '#3498db', color: 'white'}}
              >
                📄 Upload CSV
              </button>
              <button
                onClick={() => setShowVoterModal(true)}
                className="btn btn-success"
              >
                + Add Voter
              </button>
            </div>
          </div>
          <div className="card-body">
            {voters.length === 0 ? (
              <p style={{color: '#7f8c8d'}}>No voters registered yet</p>
            ) : (
              <table className="table">
                <thead>
                  <tr>
                    <th>TC Kimlik No</th>
                    <th>Docker Port</th>
                    <th>Status</th>
                    <th>Registered At</th>
                    <th>Voted At</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {voters.map(voter => (
                    <tr key={voter.id}>
                      <td><strong>{voter.voter_id}</strong></td>
                      <td style={{fontSize: '12px', fontFamily: 'monospace'}}>
                        {voter.docker_port ? `Port: ${voter.docker_port}` : 'Pending'}
                      </td>
                      <td>{getStatusBadge(voter.status)}</td>
                      <td>{new Date(voter.created_at).toLocaleString('tr-TR')}</td>
                      <td>{voter.voted_at ? new Date(voter.voted_at).toLocaleString('tr-TR') : '-'}</td>
                      <td>
                        <button
                          onClick={() => handleDeleteVoter(voter.id, voter.voter_id)}
                          className="btn btn-danger"
                          style={{padding: '6px 12px', fontSize: '12px'}}
                        >
                          Remove
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>

        {/* Add Voter Modal */}
        {showVoterModal && (
          <div className="modal-overlay" onClick={() => setShowVoterModal(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h2 className="modal-title">Add Voter</h2>
                <button className="modal-close" onClick={() => setShowVoterModal(false)}>
                  &times;
                </button>
              </div>
              <form onSubmit={handleAddVoter}>
                <div className="modal-body">
                  <div className="form-group">
                    <label>TC Kimlik Numarası *</label>
                    <input
                      type="text"
                      value={voterTcId}
                      onChange={(e) => setVoterTcId(e.target.value)}
                      placeholder="11 digit TC ID"
                      pattern="[0-9]{11}"
                      maxLength="11"
                      required
                    />
                    <small style={{color: '#7f8c8d', fontSize: '12px'}}>
                      Enter 11-digit Turkish ID number
                    </small>
                  </div>
                </div>

                <div className="modal-footer">
                  <button
                    type="button"
                    onClick={() => setShowVoterModal(false)}
                    className="btn"
                    style={{background: '#95a5a6', color: 'white'}}
                  >
                    Cancel
                  </button>
                  <button type="submit" className="btn btn-success">
                    Add Voter
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* Add Trustee Modal */}
        {showTrusteeModal && (
          <div className="modal-overlay" onClick={() => setShowTrusteeModal(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h2 className="modal-title">Add Trustee</h2>
                <button className="modal-close" onClick={() => setShowTrusteeModal(false)}>
                  &times;
                </button>
              </div>
              <form onSubmit={handleAddTrustee}>
                <div className="modal-body">
                  <div className="form-group">
                    <label>Trustee Name *</label>
                    <input
                      type="text"
                      value={trusteeName}
                      onChange={(e) => setTrusteeName(e.target.value)}
                      placeholder="Enter trustee name"
                      required
                    />
                  </div>

                  <div className="form-group" style={{marginTop: '15px'}}>
                    <label>Connection Type *</label>
                    <select
                      value={trusteeDockerType}
                      onChange={(e) => setTrusteeDockerType(e.target.value)}
                      style={{
                        width: '100%',
                        padding: '8px 12px',
                        border: '1px solid #ddd',
                        borderRadius: '4px',
                        fontSize: '14px'
                      }}
                    >
                      <option value="auto">Docker Auto (System will assign port)</option>
                      <option value="manual">Manual IP Address</option>
                    </select>
                  </div>

                  {trusteeDockerType === 'manual' && (
                    <div className="form-group" style={{marginTop: '15px'}}>
                      <label>IP Address *</label>
                      <input
                        type="text"
                        value={trusteeIpAddress}
                        onChange={(e) => setTrusteeIpAddress(e.target.value)}
                        placeholder="e.g., 192.168.1.100:8080"
                        required={trusteeDockerType === 'manual'}
                      />
                      <small style={{color: '#7f8c8d', fontSize: '12px', display: 'block', marginTop: '5px'}}>
                        Enter IP address with port (e.g., 192.168.1.100:8080)
                      </small>
                    </div>
                  )}
                </div>

                <div className="modal-footer">
                  <button
                    type="button"
                    onClick={() => setShowTrusteeModal(false)}
                    className="btn"
                    style={{background: '#95a5a6', color: 'white'}}
                  >
                    Cancel
                  </button>
                  <button type="submit" className="btn btn-primary">
                    Add Trustee
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* CSV Upload Modal */}
        {showCsvModal && (
          <div className="modal-overlay" onClick={() => setShowCsvModal(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h2 className="modal-title">Upload Voters CSV</h2>
                <button className="modal-close" onClick={() => setShowCsvModal(false)}>
                  &times;
                </button>
              </div>
              <form onSubmit={handleCsvUpload}>
                <div className="modal-body">
                  <div className="form-group">
                    <label>CSV File *</label>
                    <input
                      type="file"
                      accept=".csv"
                      onChange={(e) => setCsvFile(e.target.files[0])}
                      required
                    />
                    <small style={{color: '#7f8c8d', fontSize: '12px', display: 'block', marginTop: '8px'}}>
                      CSV format: <code>election_id,tc_id</code>
                      <br />
                      Example: <code>{id},12345678901</code>
                    </small>
                  </div>

                  {csvUploadResult && (
                    <div style={{
                      marginTop: '15px',
                      padding: '12px',
                      background: csvUploadResult.failed > 0 ? '#fff3cd' : '#d4edda',
                      border: `1px solid ${csvUploadResult.failed > 0 ? '#ffc107' : '#28a745'}`,
                      borderRadius: '4px'
                    }}>
                      <p style={{margin: '0 0 8px 0', fontWeight: 'bold'}}>
                        Upload Results:
                      </p>
                      <p style={{margin: '4px 0'}}>
                        <strong>Imported:</strong> {csvUploadResult.imported} voters
                      </p>
                      <p style={{margin: '4px 0'}}>
                        <strong>Failed:</strong> {csvUploadResult.failed} rows
                      </p>
                      {csvUploadResult.errors && csvUploadResult.errors.length > 0 && (
                        <div style={{marginTop: '10px'}}>
                          <strong>Errors:</strong>
                          <ul style={{margin: '5px 0 0 20px', fontSize: '12px'}}>
                            {csvUploadResult.errors.map((error, idx) => (
                              <li key={idx}>{error}</li>
                            ))}
                          </ul>
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="modal-footer">
                  <button
                    type="button"
                    onClick={() => {
                      setShowCsvModal(false)
                      setCsvUploadResult(null)
                      setCsvFile(null)
                    }}
                    className="btn"
                    style={{background: '#95a5a6', color: 'white'}}
                  >
                    Close
                  </button>
                  <button
                    type="submit"
                    className="btn"
                    style={{background: '#3498db', color: 'white'}}
                  >
                    Upload CSV
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* Crypto Parameters Modal */}
        {showCryptoModal && cryptoParams && (
          <div className="modal-overlay" onClick={() => setShowCryptoModal(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()} style={{maxWidth: '800px'}}>
              <div className="modal-header">
                <h2 className="modal-title">🔐 Cryptographic Parameters</h2>
                <button className="modal-close" onClick={() => setShowCryptoModal(false)}>
                  &times;
                </button>
              </div>
              <div className="modal-body">
                <div style={{marginBottom: '20px'}}>
                  <div style={{
                    display: 'grid',
                    gridTemplateColumns: '1fr',
                    gap: '15px'
                  }}>
                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Election ID:</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '12px', wordBreak: 'break-all'}}>
                        {cryptoParams.election_id}
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Security Level:</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '12px'}}>
                        {cryptoParams.security_level} bits
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Prime Order (q):</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '11px', wordBreak: 'break-all', color: '#6c757d'}}>
                        {cryptoParams.prime_order}
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Generator G1:</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '11px', wordBreak: 'break-all', color: '#6c757d'}}>
                        {cryptoParams.g1}
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Generator G2:</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '11px', wordBreak: 'break-all', color: '#6c757d'}}>
                        {cryptoParams.g2}
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#f8f9fa', borderRadius: '6px', border: '1px solid #dee2e6'}}>
                      <strong style={{color: '#495057', fontSize: '13px'}}>Hash Point H1:</strong>
                      <p style={{margin: '5px 0 0 0', fontFamily: 'monospace', fontSize: '11px', wordBreak: 'break-all', color: '#6c757d'}}>
                        {cryptoParams.h1}
                      </p>
                    </div>

                    <div style={{padding: '12px', background: '#fff3cd', borderRadius: '6px', border: '1px solid #ffc107'}}>
                      <strong style={{color: '#856404', fontSize: '13px'}}>⚠️ Pairing Parameters (Type A Curve):</strong>
                      <pre style={{
                        margin: '8px 0 0 0',
                        padding: '10px',
                        background: '#fff',
                        borderRadius: '4px',
                        fontSize: '10px',
                        overflow: 'auto',
                        maxHeight: '150px',
                        border: '1px solid #dee2e6'
                      }}>
                        {cryptoParams.pairing_params}
                      </pre>
                    </div>
                  </div>
                </div>

                <div style={{
                  padding: '12px',
                  background: '#d1ecf1',
                  border: '1px solid #bee5eb',
                  borderRadius: '4px',
                  fontSize: '12px',
                  color: '#0c5460'
                }}>
                  <strong>ℹ️ Info:</strong> These parameters are generated using PBC (Pairing-Based Cryptography) library
                  with Type A symmetric pairing curves. All participants (TTP, Trustees, Voters) use these same parameters.
                </div>
              </div>

              <div className="modal-footer">
                <button
                  type="button"
                  onClick={() => setShowCryptoModal(false)}
                  className="btn"
                  style={{background: '#8e44ad', color: 'white'}}
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default ElectionDetail
