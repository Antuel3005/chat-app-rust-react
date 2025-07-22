use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use warp::Filter;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: String,
    username: String,
    message: String,
    timestamp: u64,
    is_ai: bool,
    session_id: String,
}

#[derive(Debug, Clone)]
struct DatabaseMessage {
    id: String,
    username: String,
    message: String,
    timestamp: DateTime<Utc>,
    is_ai: bool,
    session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

type Users = Arc<RwLock<HashMap<String, broadcast::Sender<ChatMessage>>>>;
type UserSessions = Arc<RwLock<HashMap<String, String>>>; // Maps user_id to session_id

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt::init();
    
    // Optional: Set Gemini API URL
    let gemini_url = env::var("GEMINI_API_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent".to_string());
    
    println!("Using Gemini API at: {}", gemini_url);
    
    // Initialize PostgreSQL database
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL database");
    
    println!("Connected to PostgreSQL database");
    
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id VARCHAR(36) PRIMARY KEY,
            username VARCHAR(255) NOT NULL,
            message TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL,
            is_ai BOOLEAN NOT NULL DEFAULT FALSE,
            session_id VARCHAR(36) NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create messages table");
    
    // Create index on session_id for better query performance
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)"
    )
    .execute(&pool)
    .await
    .expect("Failed to create session_id index");
    
    println!("Database initialized successfully");
    
    let users = Users::default();
    let user_sessions = UserSessions::default();
    let (tx, _rx) = broadcast::channel(100);
    let broadcast_tx = Arc::new(tx);
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);
    
    // WebSocket route with user authentication
    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_users(users.clone()))
        .and(with_user_sessions(user_sessions.clone()))
        .and(with_broadcast(broadcast_tx.clone()))
        .and(with_db(pool.clone()))
        .map(|ws: warp::ws::Ws, query: std::collections::HashMap<String, String>, users, user_sessions, broadcast_tx, pool| {
            ws.on_upgrade(move |socket| handle_websocket(socket, query, users, user_sessions, broadcast_tx, pool))
        });
    
    // Static files route for serving React app
    let static_files = warp::path("static")
        .and(warp::fs::dir("./frontend/build/static"));
    
    // Serve index.html for all other routes (SPA routing)
    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./frontend/build/index.html"));
    
    let routes = websocket
        .or(static_files)
        .or(index)
        .with(cors);
    
    // Get port from environment variable (Cloud Run sets this)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    println!("Chat server starting on http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
    
    Ok(())
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn with_user_sessions(user_sessions: UserSessions) -> impl Filter<Extract = (UserSessions,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || user_sessions.clone())
}

fn with_broadcast(
    broadcast_tx: Arc<broadcast::Sender<ChatMessage>>,
) -> impl Filter<Extract = (Arc<broadcast::Sender<ChatMessage>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || broadcast_tx.clone())
}

fn with_db(
    pool: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn handle_websocket(
    ws: warp::ws::WebSocket,
    query: std::collections::HashMap<String, String>,
    _users: Users,
    user_sessions: UserSessions,
    broadcast_tx: Arc<broadcast::Sender<ChatMessage>>,
    pool: PgPool,
) {
    // Extract user info from query parameters
    let username = match query.get("username") {
        Some(name) => name.clone(),
        None => {
            eprintln!("WebSocket connection rejected: missing username");
            return;
        }
    };
    
    let user_email = match query.get("email") {
        Some(email) => email.clone(),
        None => {
            eprintln!("WebSocket connection rejected: missing email");
            return;
        }
    };
    
    // Create or get user session (each user gets their own private session)
    let session_id = Uuid::new_v4().to_string();
    let user_id = user_email.clone(); // Use email as unique user identifier
    
    // Store user session mapping
    {
        let mut sessions = user_sessions.write().await;
        sessions.insert(user_id.clone(), session_id.clone());
    }
    
    let (mut ws_tx, mut ws_rx) = ws.split();
    
    println!("User {} ({}) connected with session {}", username, user_email, session_id);
    
    // Send recent messages from this user's session to the user
    if let Ok(recent_messages) = get_recent_messages_by_session(&pool, &session_id, 50).await {
        for msg in recent_messages {
            let chat_msg = ChatMessage {
                id: msg.id,
                username: msg.username,
                message: msg.message,
                timestamp: msg.timestamp.timestamp_millis() as u64,
                is_ai: msg.is_ai,
                session_id: msg.session_id,
            };
            let json = serde_json::to_string(&chat_msg).unwrap();
            if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                return;
            }
        }
    }
    
    // Subscribe to broadcast channel
    let mut rx = broadcast_tx.subscribe();
    
    // Spawn task to handle incoming messages from this user
    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let session_id_clone = session_id.clone();
    let username_clone = username.clone();
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                            // Ensure user messages are not marked as AI and set session_id
                            chat_msg.is_ai = false;
                            chat_msg.session_id = session_id_clone.clone();
                            chat_msg.username = username_clone.clone();
                            
                            // Save user message to database
                             let _ = save_message_to_db(&pool_clone, &chat_msg).await;
                            
                            // Broadcast the user message (only to this session)
                            let _ = broadcast_tx_clone.send(chat_msg.clone());
                            
                            // Check if AI should respond
                             if should_ai_respond(&chat_msg.message) {
                                 // Get recent conversation context from this user's session
                                 let context_messages = get_recent_messages_by_session(&pool_clone, &session_id_clone, 10).await.unwrap_or_default();
                                 let ai_response = get_ai_response_with_context(&chat_msg.message, &context_messages, &chat_msg.username).await;
                                 if let Some(response_text) = ai_response {
                                    let ai_message = ChatMessage {
                                        id: Uuid::new_v4().to_string(),
                                        username: "AI Assistant".to_string(),
                                        message: response_text,
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis() as u64,
                                        is_ai: true,
                                        session_id: session_id_clone.clone(),
                                    };
                                    
                                    // Save AI message to database
                                    let _ = save_message_to_db(&pool_clone, &ai_message).await;
                                    
                                    // Small delay to make AI response feel more natural
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                    let _ = broadcast_tx_clone.send(ai_message);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error for user {}: {}", user_id_clone, e);
                    break;
                }
            }
        }
    });
    
    // Handle outgoing messages to this user (only messages from their session)
    while let Ok(msg) = rx.recv().await {
        // Only send messages that belong to this user's session
        if msg.session_id == session_id {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                break;
            }
        }
    }
    
    // Clean up user session when disconnected
    {
        let mut sessions = user_sessions.write().await;
        sessions.remove(&user_id);
    }
    
    println!("User {} ({}) disconnected from session {}", username, user_email, session_id);
}

// Function to determine if AI should respond to a message
fn should_ai_respond(message: &str) -> bool {
    let message_lower = message.to_lowercase();
    
    // AI responds to:
    // 1. Messages that mention "ai", "bot", "assistant"
    // 2. Questions (containing "?", "what", "how", "why", "when", "where", "who")
    // 3. Greetings
    // 4. Help requests
    
    let ai_triggers = [
        "ai", "bot", "assistant", "help", "hello", "hi", "hey", 
        "what", "how", "why", "when", "where", "who", "can you",
        "please", "thanks", "thank you"
    ];
    
    // Check for question marks or trigger words
    message.contains('?') || 
    ai_triggers.iter().any(|&trigger| message_lower.contains(trigger))
}

// Function to get AI response from Gemini API
async fn get_ai_response_with_context(user_message: &str, context_messages: &[DatabaseMessage], current_user: &str) -> Option<String> {
    // Gemini API endpoint
    let gemini_url = env::var("GEMINI_API_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent".to_string());
    
    let api_key = env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable must be set");
    
    let client = reqwest::Client::new();
    
    // Build conversation context
    let mut conversation_text = format!("You are a helpful AI assistant in a group chat. The user '{}' just sent a message. Respond in a friendly, conversational way. Keep your response concise (1-2 sentences max) and engaging. Be helpful and natural.\n\n", current_user);
    
    // Add conversation context
    if !context_messages.is_empty() {
        conversation_text.push_str("Recent conversation:\n");
        for msg in context_messages.iter().rev().take(5) { // Last 5 messages for context
            if msg.is_ai {
                conversation_text.push_str(&format!("AI: {}\n", msg.message));
            } else {
                conversation_text.push_str(&format!("{}: {}\n", msg.username, msg.message));
            }
        }
    }
    
    // Add current user message
    conversation_text.push_str(&format!("\nCurrent message from {}: {}\n\nPlease respond:", current_user, user_message));
    
    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: conversation_text,
            }],
        }],
    };
    
    match client
        .post(&format!("{}?key={}", gemini_url, api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GeminiResponse>().await {
                    Ok(gemini_response) => {
                        if let Some(candidate) = gemini_response.candidates.first() {
                            if let Some(part) = candidate.content.parts.first() {
                                return Some(part.text.clone());
                            }
                        }
                        eprintln!("No valid response content from Gemini API");
                        None
                    }
                    Err(e) => {
                        eprintln!("Failed to parse Gemini API response: {}", e);
                        None
                    }
                }
            } else {
                eprintln!("Gemini API request failed with status: {}", response.status());
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to send request to Gemini API: {}", e);
            None
        }
    }
}

// Function to save message to database
async fn save_message_to_db(pool: &PgPool, message: &ChatMessage) -> Result<(), sqlx::Error> {
    let timestamp = DateTime::<Utc>::from_timestamp_millis(message.timestamp as i64)
        .unwrap_or_else(|| Utc::now());
    
    sqlx::query(
        "INSERT INTO messages (id, username, message, timestamp, is_ai, session_id) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&message.id)
    .bind(&message.username)
    .bind(&message.message)
    .bind(timestamp)
    .bind(message.is_ai)
    .bind(&message.session_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

// Function to get recent messages from database by session
async fn get_recent_messages_by_session(pool: &PgPool, session_id: &str, limit: i32) -> Result<Vec<DatabaseMessage>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, message, timestamp, is_ai, session_id FROM messages WHERE session_id = $1 ORDER BY timestamp DESC LIMIT $2"
    )
    .bind(session_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    let mut messages = Vec::new();
    for row in rows {
        let message = DatabaseMessage {
            id: row.get("id"),
            username: row.get("username"),
            message: row.get("message"),
            timestamp: row.get("timestamp"),
            is_ai: row.get("is_ai"),
            session_id: row.get("session_id"),
        };
        messages.push(message);
    }
    
    // Reverse to get chronological order (oldest first)
    messages.reverse();
    Ok(messages)
}

// Function to get recent messages from database (legacy - kept for compatibility)
async fn get_recent_messages(pool: &PgPool, limit: i32) -> Result<Vec<DatabaseMessage>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, message, timestamp, is_ai, session_id FROM messages ORDER BY timestamp DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    let mut messages = Vec::new();
    for row in rows {
        let message = DatabaseMessage {
            id: row.get("id"),
            username: row.get("username"),
            message: row.get("message"),
            timestamp: row.get("timestamp"),
            is_ai: row.get("is_ai"),
            session_id: row.get("session_id"),
        };
        messages.push(message);
    }
    
    // Reverse to get chronological order (oldest first)
    messages.reverse();
    Ok(messages)
}