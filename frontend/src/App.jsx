import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import Login from './pages/Login'
import Dashboard from './pages/Dashboard'
import Elections from './pages/Elections'
import ElectionDetail from './pages/ElectionDetail'
import Logs from './pages/Logs'
import Monitoring from './pages/Monitoring'

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false)

  useEffect(() => {
    const token = localStorage.getItem('token')
    if (token) {
      setIsAuthenticated(true)
    }
  }, [])

  const handleLogin = () => {
    setIsAuthenticated(true)
  }

  const handleLogout = () => {
    localStorage.removeItem('token')
    setIsAuthenticated(false)
  }

  return (
    <Router>
      <Routes>
        <Route
          path="/login"
          element={
            isAuthenticated ?
            <Navigate to="/dashboard" /> :
            <Login onLogin={handleLogin} />
          }
        />
        <Route
          path="/dashboard"
          element={
            isAuthenticated ?
            <Dashboard onLogout={handleLogout} /> :
            <Navigate to="/login" />
          }
        />
        <Route
          path="/elections"
          element={
            isAuthenticated ?
            <Elections onLogout={handleLogout} /> :
            <Navigate to="/login" />
          }
        />
        <Route
          path="/elections/:id"
          element={
            isAuthenticated ?
            <ElectionDetail onLogout={handleLogout} /> :
            <Navigate to="/login" />
          }
        />
        <Route
          path="/logs"
          element={
            isAuthenticated ?
            <Logs onLogout={handleLogout} /> :
            <Navigate to="/login" />
          }
        />
        <Route
          path="/monitoring"
          element={
            isAuthenticated ?
            <Monitoring onLogout={handleLogout} /> :
            <Navigate to="/login" />
          }
        />
        <Route path="/" element={<Navigate to="/dashboard" />} />
      </Routes>
    </Router>
  )
}

export default App
