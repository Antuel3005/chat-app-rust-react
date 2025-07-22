import React, { useState, useEffect, useRef } from 'react';
import { GoogleOAuthProvider, GoogleLogin, googleLogout } from '@react-oauth/google';
import './App.css';

const CLIENT_ID = process.env.REACT_APP_GOOGLE_CLIENT_ID;

function App() {
  const [messages, setMessages] = useState([]);
  const [inputMessage, setInputMessage] = useState('');
  const [user, setUser] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const wsRef = useRef(null);
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const connectWebSocketWithUser = (userInfo) => {
    if (!userInfo || !userInfo.name || !userInfo.email) return;
    
    // Create WebSocket connection with user authentication
    // Use environment variable for backend URL, fallback to localhost for development
    const backendUrl = process.env.REACT_APP_BACKEND_URL || 'ws://localhost:3001';
    const wsUrl = `${backendUrl}/ws?username=${encodeURIComponent(userInfo.name)}&email=${encodeURIComponent(userInfo.email)}`;
    console.log('Connecting to:', wsUrl);
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      console.log('Connected to chat server');
    };

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      setMessages(prev => [...prev, message]);
    };

    ws.onclose = () => {
      setIsConnected(false);
      console.log('Disconnected from chat server');
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
    };
  };

  const onLoginSuccess = (credentialResponse) => {
    // Decode the JWT token to get user info
    const decoded = JSON.parse(atob(credentialResponse.credential.split('.')[1]));
    console.log('Login Success:', decoded);
    const userInfo = {
      name: decoded.name,
      email: decoded.email,
      imageUrl: decoded.picture
    };
    setUser(userInfo);
    setIsAuthenticated(true);
    // Connect WebSocket with userInfo directly instead of relying on state
    connectWebSocketWithUser(userInfo);
  };

  const onLoginFailure = () => {
    console.log('Login Failed');
  };

  const handleLogout = () => {
    console.log('Logout Success');
    googleLogout();
    setUser(null);
    setIsAuthenticated(false);
    setIsConnected(false);
    if (wsRef.current) {
      wsRef.current.close();
    }
  };

  const connectWebSocket = (username) => {
    if (!username || !user) return;
    connectWebSocketWithUser(user);
  };

  const sendMessage = () => {
    if (!inputMessage.trim() || !isConnected || !user) return;

    const message = {
      id: Date.now().toString(),
      username: user.name,
      message: inputMessage.trim(),
      timestamp: Date.now(),
      is_ai: false,
      session_id: '' // Will be set by backend
    };

    wsRef.current.send(JSON.stringify(message));
    setInputMessage('');
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && isAuthenticated) {
      sendMessage();
    }
  };

  const formatTime = (timestamp) => {
    return new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  if (!isAuthenticated) {
    return (
      <div className="app">
        <div className="username-container">
          <h1>🤖 Private AI Assistant</h1>
          <div className="oauth-container">
            <p>Sign in with your Google account to start your private AI chat session</p>
            <div className="features-info">
              <p>✨ Your own private conversation with AI</p>
              <p>🔒 No other users can see your messages</p>
              <p>💬 Personalized responses just for you</p>
            </div>
            <GoogleLogin
               onSuccess={onLoginSuccess}
               onError={onLoginFailure}
               theme="outline"
               size="large"
               text="signin_with"
               shape="rectangular"
             />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <div className="chat-container">
        <div className="chat-header">
          <div className="header-left">
            <h1>🤖 Private AI Assistant</h1>
            <div className="user-info">
              <img src={user?.imageUrl} alt="Profile" className="profile-image" />
              <span className="user-name">{user?.name}</span>
              <span className="session-info">• Private Session</span>
            </div>
          </div>
          <div className="header-right">
            <div className="connection-status">
              <span className={`status-indicator ${isConnected ? 'connected' : 'disconnected'}`}></span>
              {isConnected ? 'Connected' : 'Disconnected'}
            </div>
            <button
               onClick={handleLogout}
               className="logout-button"
             >
               Logout
             </button>
          </div>
        </div>
        
        <div className="messages-container">
          {messages.map((msg) => (
            <div key={msg.id} className={`message ${msg.username === user?.name ? 'own-message' : ''} ${msg.is_ai ? 'ai-message' : ''}`}>
              <div className="message-header">
                <span className={`username ${msg.is_ai ? 'ai-username' : ''}`}>
                  {msg.is_ai && '🤖 '}{msg.username}
                </span>
                <span className="timestamp">{formatTime(msg.timestamp)}</span>
              </div>
              <div className="message-content">{msg.message}</div>
            </div>
          ))}
          <div ref={messagesEndRef} />
        </div>
        
        <div className="input-container">
          <input
            type="text"
            placeholder="Type your message..."
            value={inputMessage}
            onChange={(e) => setInputMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            disabled={!isConnected}
          />
          <button onClick={sendMessage} disabled={!isConnected || !inputMessage.trim()}>
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

function AppWithProvider() {
  return (
    <GoogleOAuthProvider clientId={CLIENT_ID}>
      <App />
    </GoogleOAuthProvider>
  );
}

export default AppWithProvider;