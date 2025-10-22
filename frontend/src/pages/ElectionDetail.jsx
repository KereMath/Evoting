import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import Navbar from '../components/Navbar'
import { getElection, getVoters, createVoter, deleteVoter, getTrustees, createTrustee, uploadVotersCSV, setupElection, setupCrypto, startKeygen, getKeygenStatus, advanceToDIDPhase } from '../services/api'

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
  const [keygenLoading, setKeygenLoading] = useState(false)
  const [keygenStatus, setKeygenStatus] = useState(null)

  useEffect(() => {
    loadElectionData()
  }, [id])

  // Auto-trigger keygen when phase 5 is reached (crypto setup complete)
  useEffect(() => {
    if (election && election.phase === 5 && !keygenLoading) {
      console.log('Auto-triggering keygen...')
      autoStartKeygen()
    }
  }, [election?.phase])

  // Auto-poll DKG status when in key generation phase OR voter DID status in phase 7
  useEffect(() => {
    let pollInterval = null

    if (election && election.phase === 7) {
      // Poll voter DID status every 3 seconds
      pollInterval = setInterval(() => {
        loadElectionData()
      }, 3000)
    } else if (election && election.phase === 6 && election.status === 'key_generation') {
      // Start polling immediately
      const pollStatus = async () => {
        try {
          const statusRes = await getKeygenStatus(id)
          setKeygenStatus(statusRes.data)

          // If completed, stop polling, reload election data, and auto-advance to DID phase
          if (statusRes.data.status === 'completed') {
            if (pollInterval) clearInterval(pollInterval)
            console.log('✅ Keygen complete! Auto-advancing to DID phase...')
            await handleAdvanceToDID()
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

  const autoSetupElection = async () => {
    setSetupLoading(true)
    try {
      console.log('🚀 Auto-triggering election setup...')
      const response = await setupElection(id)
      console.log('✅ Setup complete:', response.data.message)

      // Immediately trigger crypto setup after election setup
      setTimeout(async () => {
        await autoCryptoSetup()
      }, 2000)
    } catch (error) {
      console.error('Failed to auto-setup election:', error)
    } finally {
      setSetupLoading(false)
    }
  }

  const autoCryptoSetup = async () => {
    setCryptoSetupLoading(true)
    try {
      console.log('🔐 Auto-triggering crypto setup...')
      const response = await setupCrypto(id, 256)
      console.log('✅ Crypto setup complete:', response.data.message)
      loadElectionData()
    } catch (error) {
      console.error('Failed to auto-setup crypto:', error)
    } finally {
      setCryptoSetupLoading(false)
    }
  }

  const autoStartKeygen = async () => {
    setKeygenLoading(true)
    try {
      console.log('🔑 Auto-triggering keygen...')
      const response = await startKeygen(id)
      console.log('✅ Keygen started:', response.data.message)
      loadElectionData()
    } catch (error) {
      console.error('Failed to auto-start keygen:', error)
    } finally {
      setKeygenLoading(false)
    }
  }

  const handleAdvanceToDID = async () => {
    try {
      await advanceToDIDPhase(id)
      loadElectionData()
    } catch (error) {
      console.error('Failed to advance to DID phase:', error)
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

        {/* Simplified Phase Status */}
        <div className="card" style={{marginTop: '20px'}}>
          <div className="card-header">
            <h2 className="card-title">Election Status</h2>
          </div>
          <div className="card-body">
            {election.phase < 7 && (
              <div style={{padding: '15px', background: '#fff3cd', border: '1px solid #ffc107', borderRadius: '4px'}}>
                <strong style={{color: '#856404', fontSize: '16px'}}>
                  {election.phase === 1 && '📋 Phase 1: Add Trustees'}
                  {election.phase === 2 && '👥 Phase 2: Add Voters'}
                  {election.phase === 3 && '✅ Phase 3: Ready to Start'}
                  {election.phase === 4 && '🔐 Phase 4: Crypto setup in progress...'}
                  {election.phase === 5 && '🔑 Phase 5: Keygen starting...'}
                  {election.phase === 6 && '⏳ Phase 6: Key generation in progress...'}
                </strong>
                <p style={{fontSize: '12px', color: '#856404', margin: '8px 0 0 0'}}>
                  {election.phase === 1 && `Add ${election.total_trustees - trustees.length} more trustees to continue`}
                  {election.phase === 2 && 'Add voters to the election'}
                  {election.phase === 3 && `${trustees.length} trustees and ${voters.length} voters added. Click "Start Setup" when ready.`}
                  {election.phase === 4 && 'System is automatically generating cryptographic parameters...'}
                  {election.phase === 5 && 'System is automatically starting distributed key generation...'}
                  {election.phase === 6 && keygenStatus && `DKG Step ${keygenStatus.current_step}/7 - ${keygenStatus.trustees_ready.filter(t => t.status === 'completed').length}/${keygenStatus.total_trustees} trustees ready`}
                </p>
                {election.phase === 3 && (
                  <button
                    onClick={autoSetupElection}
                    disabled={setupLoading}
                    className="btn btn-primary"
                    style={{marginTop: '15px', width: '100%', padding: '12px', fontSize: '16px', background: '#28a745', color: 'white'}}
                  >
                    {setupLoading ? 'Starting Setup...' : '🚀 Start Setup'}
                  </button>
                )}
              </div>
            )}

            {election.phase === 7 && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '15px', background: '#d4edda', border: '1px solid #28a745', borderRadius: '4px', marginBottom: '15px'}}>
                  <strong style={{color: '#155724', fontSize: '16px'}}>✅ Setup Complete - DID Generation Phase</strong>
                  <p style={{fontSize: '12px', color: '#155724', margin: '5px 0 0 0'}}>
                    Voters can now generate their DIDs using the voter UI ports below
                  </p>
                </div>
              </div>
            )}

            {election.phase === 8 && (
              <div style={{marginTop: '20px'}}>
                <div style={{padding: '15px', background: '#e7f3ff', border: '1px solid #007bff', borderRadius: '4px', marginBottom: '15px'}}>
                  <strong style={{color: '#004085', fontSize: '16px'}}>✅ Tüm voterlar prepare blindsign requestini gönderdi, trusteelerini bekliyorlar</strong>
                  <p style={{fontSize: '12px', color: '#004085', margin: '5px 0 0 0'}}>
                    All voters completed PrepareBlindSign and are waiting for trustees to issue credentials.
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>


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

      </div>
    </div>
  )
}

export default ElectionDetail
