import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add token to requests if available
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Auth
export const login = (username, password) =>
  api.post('/api/auth/login', { username, password });

// Elections
export const getElections = () =>
  api.get('/api/elections');

export const getElection = (id) =>
  api.get(`/api/elections/${id}`);

export const createElection = (data) =>
  api.post('/api/elections', data);

export const deleteElection = (id) =>
  api.delete(`/api/elections/${id}`);

// Voters
export const getVoters = (electionId = null) =>
  api.get('/api/voters', { params: electionId ? { election_id: electionId } : {} });

export const createVoter = (data) =>
  api.post('/api/voters', data);

export const deleteVoter = (id) =>
  api.delete(`/api/voters/${id}`);

// Trustees
export const getTrustees = () =>
  api.get('/api/trustees');

export const createTrustee = (data) =>
  api.post('/api/trustees', data);

// Events (Logs)
export const getEvents = (params = {}) =>
  api.get('/api/events', { params });

// CSV Upload
export const uploadVotersCSV = (formData) =>
  api.post('/api/voters/upload-csv', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });

// Monitoring
export const getNetworkTopology = () =>
  api.get('/api/monitoring/topology');

// Orchestration
export const setupElection = (electionId) =>
  api.post(`/api/elections/${electionId}/setup`);

// Crypto Setup
export const setupCrypto = (electionId, securityLevel = 256) =>
  api.post(`/api/elections/${electionId}/crypto-setup`, { security_level: securityLevel });

// Get Crypto Parameters
export const getCryptoParameters = (electionId) =>
  api.get(`/api/crypto/parameters/${electionId}`);

export default api;
