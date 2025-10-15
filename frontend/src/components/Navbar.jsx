import { Link } from 'react-router-dom'

function Navbar({ onLogout }) {
  return (
    <nav className="navbar">
      <div className="navbar-brand">E-Voting Admin Panel</div>
      <ul className="navbar-nav">
        <li><Link to="/dashboard">Dashboard</Link></li>
        <li><Link to="/elections">Elections</Link></li>
        <li><Link to="/monitoring">Monitoring</Link></li>
        <li><Link to="/logs">System Logs</Link></li>
      </ul>
      <button onClick={onLogout} className="navbar-logout">
        Logout
      </button>
    </nav>
  )
}

export default Navbar
