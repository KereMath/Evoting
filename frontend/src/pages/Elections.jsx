import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import Navbar from '../components/Navbar'
import { getElections, createElection, deleteElection } from '../services/api'

function Elections({ onLogout }) {
  const [elections, setElections] = useState([])
  const [showModal, setShowModal] = useState(false)
  const [loading, setLoading] = useState(true)
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    threshold: 3,
    total_trustees: 5
  })

  useEffect(() => {
    loadElections()
  }, [])

  const loadElections = async () => {
    try {
      const response = await getElections()
      setElections(response.data)
      setLoading(false)
    } catch (error) {
      console.error('Failed to load elections:', error)
      setLoading(false)
    }
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    try {
      await createElection(formData)
      setShowModal(false)
      setFormData({
        name: '',
        description: '',
        threshold: 3,
        total_trustees: 5
      })
      loadElections()
    } catch (error) {
      console.error('Failed to create election:', error)
      alert('Failed to create election')
    }
  }

  const handleDelete = async (id, name) => {
    if (window.confirm(`Are you sure you want to delete election "${name}"?`)) {
      try {
        await deleteElection(id)
        loadElections()
      } catch (error) {
        console.error('Failed to delete election:', error)
        alert('Failed to delete election')
      }
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

  return (
    <div>
      <Navbar onLogout={onLogout} />
      <div className="container">
        <div className="card-header" style={{marginBottom: '20px'}}>
          <h1 className="card-title">Elections</h1>
          <button
            onClick={() => setShowModal(true)}
            className="btn btn-primary"
          >
            + Create Election
          </button>
        </div>

        {elections.length === 0 ? (
          <div className="card">
            <p style={{textAlign: 'center', color: '#7f8c8d', padding: '40px'}}>
              No elections yet. Create one to get started!
            </p>
          </div>
        ) : (
          <div className="card">
            <table className="table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Description</th>
                  <th>Threshold</th>
                  <th>Total Trustees</th>
                  <th>Status</th>
                  <th>Created At</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {elections.map(election => (
                  <tr key={election.id}>
                    <td><strong>{election.name}</strong></td>
                    <td>{election.description || '-'}</td>
                    <td>{election.threshold}</td>
                    <td>{election.total_trustees}</td>
                    <td>{getStatusBadge(election.status)}</td>
                    <td>{new Date(election.created_at).toLocaleDateString('tr-TR')}</td>
                    <td>
                      <div style={{display: 'flex', gap: '10px'}}>
                        <Link to={`/elections/${election.id}`}>
                          <button className="btn btn-primary" style={{padding: '6px 12px', fontSize: '12px'}}>
                            Manage
                          </button>
                        </Link>
                        <button
                          onClick={() => handleDelete(election.id, election.name)}
                          className="btn btn-danger"
                          style={{padding: '6px 12px', fontSize: '12px'}}
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {showModal && (
          <div className="modal-overlay" onClick={() => setShowModal(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h2 className="modal-title">Create New Election</h2>
                <button className="modal-close" onClick={() => setShowModal(false)}>
                  &times;
                </button>
              </div>
              <form onSubmit={handleSubmit}>
                <div className="modal-body">
                  <div className="form-group">
                    <label>Election Name *</label>
                    <input
                      type="text"
                      value={formData.name}
                      onChange={(e) => setFormData({...formData, name: e.target.value})}
                      required
                    />
                  </div>

                  <div className="form-group">
                    <label>Description</label>
                    <textarea
                      value={formData.description}
                      onChange={(e) => setFormData({...formData, description: e.target.value})}
                    />
                  </div>

                  <div className="form-group">
                    <label>Threshold (t) *</label>
                    <input
                      type="number"
                      min="1"
                      value={formData.threshold}
                      onChange={(e) => setFormData({...formData, threshold: parseInt(e.target.value)})}
                      required
                    />
                    <small style={{color: '#7f8c8d', fontSize: '12px'}}>
                      Minimum number of trustees needed for operations
                    </small>
                  </div>

                  <div className="form-group">
                    <label>Total Trustees (n) *</label>
                    <input
                      type="number"
                      min={formData.threshold}
                      value={formData.total_trustees}
                      onChange={(e) => setFormData({...formData, total_trustees: parseInt(e.target.value)})}
                      required
                    />
                    <small style={{color: '#7f8c8d', fontSize: '12px'}}>
                      Total number of election authorities
                    </small>
                  </div>
                </div>

                <div className="modal-footer">
                  <button
                    type="button"
                    onClick={() => setShowModal(false)}
                    className="btn"
                    style={{background: '#95a5a6', color: 'white'}}
                  >
                    Cancel
                  </button>
                  <button type="submit" className="btn btn-primary">
                    Create Election
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

export default Elections
