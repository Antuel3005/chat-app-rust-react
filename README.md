# Chat Backend API

A real-time chat backend API built with Rust using WebSockets and PostgreSQL. This backend is designed to be deployed on Render and provides WebSocket endpoints for real-time messaging with AI integration.

## Features

- Real-time messaging using WebSockets
- PostgreSQL database for message persistence
- AI chat integration with Google Gemini
- Session-based chat rooms
- CORS enabled for cross-origin requests
- Environment-based configuration
- Optimized for cloud deployment

## Project Structure

```
.
├── Cargo.toml          # Rust dependencies
├── src/
│   └── main.rs         # WebSocket server and API
├── Dockerfile          # Container configuration
├── .dockerignore       # Docker ignore rules
├── .gitignore          # Git ignore rules
└── README.md
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- PostgreSQL database
- Environment variables configured

### Environment Variables

Create a `.env` file with the following variables:

```env
DATABASE_URL=postgresql://username:password@localhost/chatdb
GEMINI_API_KEY=your_gemini_api_key_here
GEMINI_API_URL=https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent
PORT=3001
```

### Local Development

1. **Install dependencies and run:**
   ```bash
   cargo run
   ```
   The server will start on the port specified in PORT environment variable (default: 3001)

2. **WebSocket endpoint:**
   ```
   ws://localhost:3001/ws?username=YourName&email=your@email.com
   ```

## How it Works

**Backend API (Rust):**
- Uses `warp` for the web server framework
- WebSocket connections for real-time messaging
- PostgreSQL for message persistence
- Google Gemini AI integration for intelligent responses
- Session-based chat rooms for user isolation
- CORS enabled for cross-origin frontend integration

## API Endpoints

### WebSocket Connection
```
ws://your-backend-url.onrender.com/ws?username=YourName&email=your@email.com
```

**Query Parameters:**
- `username`: Display name for the user
- `email`: Unique identifier for the user

**Message Format:**
```json
{
  "id": "uuid",
  "username": "string",
  "message": "string",
  "timestamp": 1234567890,
  "is_ai": false,
  "session_id": "uuid"
}
```

## Deployment on Render

1. **Connect your GitHub repository to Render**
2. **Set environment variables in Render dashboard:**
   - `DATABASE_URL`: PostgreSQL connection string
   - `GEMINI_API_KEY`: Your Google Gemini API key
   - `PORT`: Will be set automatically by Render
3. **Deploy using the included Dockerfile**

## Development

- Server code is in `src/main.rs`
- Modify and restart with `cargo run`
- Database tables are created automatically on startup

## Features

- ✅ Real-time WebSocket messaging
- ✅ PostgreSQL message persistence
- ✅ AI chat integration with Google Gemini
- ✅ Session-based user isolation
- ✅ CORS enabled for frontend integration
- ✅ Cloud deployment ready
- ✅ Environment-based configuration

## Frontend Integration

This backend is designed to work with any WebSocket-capable frontend. Connect to the WebSocket endpoint with the required query parameters and start sending/receiving messages in real-time.
- Message reactions

## License

MIT License