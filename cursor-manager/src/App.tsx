import { useState, useEffect } from 'react';
import { 
  CursorSession, 
  getSessions, 
  saveSession, 
  deleteSessionById,
  openProjects, 
  launchSession,
  spoofMac
} from './api';
import './styles.css';

function App() {
  const [sessions, setSessions] = useState<CursorSession[]>([]);
  const [activeTab, setActiveTab] = useState('sessions');
  const [loading, setLoading] = useState(false);
  const [formVisible, setFormVisible] = useState(false);
  const [notification, setNotification] = useState({ message: '', type: '' });
  
  // Form state
  const [sessionName, setSessionName] = useState('');
  const [projectPaths, setProjectPaths] = useState('');
  const [spoofMacAddress, setSpoofMacAddress] = useState(false);

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    setLoading(true);
    try {
      const data = await getSessions();
      setSessions(data);
    } catch (error) {
      showNotification('Failed to load sessions', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateSession = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!sessionName.trim()) {
      showNotification('Please enter a session name', 'error');
      return;
    }

    const projects = projectPaths.trim()
      ? projectPaths.split('\n').map(path => path.trim()).filter(Boolean)
      : [];

    if (projects.length === 0) {
      showNotification('Please add at least one project path', 'error');
      return;
    }

    setLoading(true);
    try {
      const result = await saveSession(sessionName, projects);
      if (result) {
        showNotification('Session created successfully', 'success');
        setFormVisible(false);
        setSessionName('');
        setProjectPaths('');
        loadSessions();
      }
    } catch (error) {
      showNotification('Failed to create session', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleLaunchSession = async (session: CursorSession) => {
    setLoading(true);
    try {
      if (spoofMacAddress) {
        const macResult = await spoofMac();
        showNotification(`MAC address spoofed: ${macResult}`, 'success');
      }
      
      await launchSession(session.name, spoofMacAddress, false);
      showNotification(`Launching session: ${session.name}`, 'success');
      
      // Also open projects using the existing function
      await openProjects(session.projects);
    } catch (error) {
      showNotification('Failed to launch session', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteSession = async (id: string) => {
    const confirmed = confirm('Are you sure you want to delete this session?');
    if (!confirmed) return;

    setLoading(true);
    try {
      const result = await deleteSessionById(id);
      if (result) {
        showNotification('Session deleted successfully', 'success');
        loadSessions();
      }
    } catch (error) {
      showNotification('Failed to delete session', 'error');
    } finally {
      setLoading(false);
    }
  };

  const showNotification = (message: string, type: string) => {
    setNotification({ message, type });
    setTimeout(() => {
      setNotification({ message: '', type: '' });
    }, 3000);
  };

  return (
    <div className="container">
      <header>
        <h1>Cursor Session Manager</h1>
        <p>Manage your Cursor IDE sessions and workspaces</p>
      </header>

      <div className="tabs">
        <button
          className={`tab-button ${activeTab === 'sessions' ? 'active' : ''}`}
          onClick={() => setActiveTab('sessions')}
        >
          Sessions
        </button>
        <button
          className={`tab-button ${activeTab === 'settings' ? 'active' : ''}`}
          onClick={() => setActiveTab('settings')}
        >
          Settings
        </button>
      </div>

      <div className={`tab-content ${activeTab === 'sessions' ? 'active' : ''}`}>
        <div className="controls">
          <h3>My Sessions</h3>
          <button className="btn btn-primary" onClick={() => setFormVisible(true)}>
            New Session
          </button>
        </div>

        {loading ? (
          <p>Loading sessions...</p>
        ) : (
          <div className="session-list-container">
            {sessions.length === 0 ? (
              <p>No sessions found. Create your first session to get started.</p>
            ) : (
              <div className="list">
                {sessions.map((session) => (
                  <div key={session.id} className="list-item">
                    <div>
                      <div className="item-name">{session.name}</div>
                      <div className="item-details">
                        Projects: {session.projects.length} • Created: {session.created_date}
                      </div>
                    </div>
                    <div className="item-actions">
                      <button
                        className="btn btn-primary"
                        onClick={() => handleLaunchSession(session)}
                      >
                        Launch
                      </button>
                      <button
                        className="btn btn-danger"
                        onClick={() => handleDeleteSession(session.id)}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <div className={`tab-content ${activeTab === 'settings' ? 'active' : ''}`}>
        <h3>Settings</h3>
        
        <div className="form-group">
          <label>
            <input
              type="checkbox"
              checked={spoofMacAddress}
              onChange={(e) => setSpoofMacAddress(e.target.checked)}
            />
            Spoof MAC address when launching sessions
          </label>
          <p className="help-text">
            This will change your device's MAC address each time you launch a session, 
            helping to avoid detection of multiple Cursor instances.
          </p>
        </div>
        
        <div className="machine-id-warning">
          <strong>Note:</strong> If you're experiencing issues with Cursor instances, you may need to
          clear machine-id or fingerprints. This functionality will be added in a future update.
        </div>
      </div>

      {/* Create Session Modal */}
      <div className={`modal ${formVisible ? 'active' : ''}`} style={{ display: formVisible ? 'block' : 'none' }}>
        <div className="modal-content">
          <div className="modal-header">
            <h3>Create New Session</h3>
            <span className="close" onClick={() => setFormVisible(false)}>&times;</span>
          </div>

          <form onSubmit={handleCreateSession}>
            <div className="form-group">
              <label htmlFor="sessionName">Session Name</label>
              <input
                type="text"
                id="sessionName"
                className="form-control"
                value={sessionName}
                onChange={(e) => setSessionName(e.target.value)}
                placeholder="e.g., MyProject"
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="projectPaths">Project Paths (one per line)</label>
              <textarea
                id="projectPaths"
                className="form-control"
                value={projectPaths}
                onChange={(e) => setProjectPaths(e.target.value)}
                rows={5}
                placeholder="/path/to/project1&#10;/path/to/project2"
                required
              ></textarea>
            </div>

            <div className="modal-footer">
              <button type="button" className="btn" onClick={() => setFormVisible(false)}>
                Cancel
              </button>
              <button type="submit" className="btn btn-primary" disabled={loading}>
                {loading ? 'Creating...' : 'Create Session'}
              </button>
            </div>
          </form>
        </div>
      </div>

      {/* Notification */}
      {notification.message && (
        <div className={`notification ${notification.type || 'info'}`}>
          {notification.message}
        </div>
      )}
    </div>
  );
}

export default App; 