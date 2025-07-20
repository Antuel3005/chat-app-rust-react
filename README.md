# Real-time Chat Application

A minimal real-time chat application built with React (frontend) and Rust (backend) using WebSockets.

## Features

- Real-time messaging using WebSockets
- Multiple users can join with a shareable link
- Clean and modern UI
- Responsive design for mobile and desktop
- Connection status indicator
- Message timestamps

## Project Structure

```
.
├── Cargo.toml          # Rust backend dependencies
├── src/
│   └── main.rs         # Rust WebSocket server
├── frontend/
│   ├── package.json    # React dependencies
│   ├── public/
│   │   └── index.html  # HTML template
│   └── src/
│       ├── App.js      # Main React component
│       ├── App.css     # Styles
│       ├── index.js    # React entry point
│       └── index.css   # Global styles
└── README.md
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (version 14 or higher)
- [npm](https://www.npmjs.com/) or [yarn](https://yarnpkg.com/)

### Installation & Running

1. **Start the Rust backend server:**
   ```bash
   cargo run
   ```
   The server will start on `http://localhost:3001`

2. **In a new terminal, install and start the React frontend:**
   ```bash
   cd frontend
   npm install
   npm start
   ```
   The React app will start on `http://localhost:3000`

3. **Open your browser and navigate to `http://localhost:3000`**

4. **Share the link with others to start chatting!**

## How it Works

1. **Backend (Rust):** 
   - Uses `tokio-tungstenite` for WebSocket handling
   - Uses `warp` for the web server
   - Broadcasts messages to all connected clients
   - Handles user connections and disconnections

2. **Frontend (React):**
   - Connects to the WebSocket server
   - Provides a clean chat interface
   - Handles real-time message display
   - Shows connection status

## Usage

1. Enter a username when prompted
2. Start sending messages
3. Share the URL with others to join the chat
4. All connected users will see messages in real-time

## Development

### Backend Development
- The Rust server code is in `src/main.rs`
- Modify and restart with `cargo run`

### Frontend Development
- React components are in `frontend/src/`
- The development server supports hot reloading
- Modify files and see changes instantly

## Future Enhancements

- User authentication
- Message persistence
- Private rooms/channels
- File sharing
- Emoji support
- User typing indicators
- Message reactions

## License

MIT License