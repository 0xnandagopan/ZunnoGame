const express = require("express");
const app = express();
const cors = require("cors");
const server = require("http").createServer(app);
const ws = require("ws");
const path = require('path');
const {addUser, removeUser, getUser, getUsersInRoom} = require("./users");
const { createClaimableBalance } = require("./diamnetService");
const logger = require('./logger');

// Set server timeout to prevent hanging connections
server.timeout = 30000; // 30 seconds

const io = require("socket.io")(server, {
    cors: {
        origin: "*",
        methods: ["GET", "POST"],
    },
    wsEngine: ws.Server,
    pingTimeout: 20000, // 20 seconds before a client is considered disconnected
    pingInterval: 10000, // Send ping every 10 seconds
    connectTimeout: 15000, // Connection timeout
    maxHttpBufferSize: 1e6, // 1MB max payload size
    transports: ['websocket', 'polling'], // Prefer WebSocket, fallback to polling
});

const PORT = process.env.PORT || 4000;

app.use(cors());
app.use(express.json());

// API endpoint for creating claimable balances
app.post("/api/create-claimable-balance", async (req, res) => {
  try {
    const { winnerAddress, gameId } = req.body;
    
    // Validate the request
    if (!winnerAddress) {
      return res.status(400).json({ error: "Winner address is required" });
    }
    
    // Create the claimable balance
    const result = await createClaimableBalance(winnerAddress);
    
    // Return success response
    res.status(200).json(result);
  } catch (error) {
    logger.error("Error creating claimable balance:", error);
    res.status(500).json({ 
      success: false, 
      error: "Failed to create claimable balance" 
    });
  }
});

if (process.env.NODE_ENV === "production") {
    app.use(express.static("frontend/build"));
    app.get("*", (req, res) => {
        res.sendFile(path.resolve(__dirname, "build", "index.html"));
    });
}

// Set up graceful shutdown
function gracefulShutdown() {
    logger.info('Shutting down gracefully...');
    server.close(() => {
        logger.info('Server closed');
        process.exit(0);
    });
    
    // Force close after 10 seconds
    setTimeout(() => {
        logger.error('Forced shutdown after timeout');
        process.exit(1);
    }, 10000);
}

process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

server.listen(PORT, () => {
    logger.info(`Server started on Port ${PORT} at ${new Date().toISOString()}`);
});

// Track active connections for monitoring
let activeConnections = 0;

// Health check endpoint for Cloud Run
app.get('/health', (req, res) => {
    res.status(200).json({
        status: 'ok',
        connections: activeConnections,
        uptime: process.uptime()
    });
});

io.on("connection", (socket) => {
    activeConnections++;
    logger.info(`User ${socket.id} connected. Active connections: ${activeConnections}`);
    io.to(socket.id).emit("server_id", socket.id);
    
    // Note: Socket timeout is already configured in the io initialization options
    // (pingTimeout, pingInterval, and connectTimeout)

    // Add room functionality
    socket.on("joinRoom", (roomId) => {
        socket.join(roomId);
        logger.info(`User ${socket.id} joined room ${roomId}`);
        io.to(roomId).emit("userJoined", socket.id);
    });

    // Add game room creation handler
    socket.on("createGameRoom", () => {
        logger.info(`Game room created by user`);
        io.emit("gameRoomCreated");
    });

    socket.on('gameStarted', (data) => {
        const { newState, cardHashMap, roomId } = data;
        logger.info(`Game started in room ${roomId}`);
        // console.log(newState)

        // Emit the gameStarted event to all clients in the room with a room-specific event name
        io.to(roomId).emit(`gameStarted-${roomId}`, { newState, cardHashMap });
    });

    // Add playCard event handler
    socket.on('playCard', (data) => {
        const { roomId, action, newState } = data;
        logger.info(`Card played in room ${roomId}`);
        // console.log('New state:', newState);

        // Broadcast the cardPlayed event to all clients in the room
        io.to(roomId).emit(`cardPlayed-${roomId}`, { action, newState });
    });

    // Add leave room functionality
    socket.on("leaveRoom", (roomId) => {
        socket.leave(roomId);
        logger.info(`User ${socket.id} left room ${roomId}`);
        io.to(roomId).emit("userLeft", socket.id);
    });

    socket.on("join", (payload, callback) => {
        let numberOfUsersInRoom = getUsersInRoom(payload.room).length;

        const { error, newUser } = addUser({
            id: socket.id,
            name: numberOfUsersInRoom === 0 ? "Player 1" : "Player 2",
            room: payload.room,
        });

        if (error) return callback(error);

        socket.join(newUser.room);

        io.to(newUser.room).emit("roomData", { room: newUser.room, users: getUsersInRoom(newUser.room) });
        socket.emit("currentUserData", { name: newUser.name });
        logger.debug(newUser)
        callback();
    });

    socket.on("initGameState", (gameState) => {
        const user = getUser(socket.id);
        if (user) io.to(user.room).emit("initGameState", gameState);
    });

    socket.on("updateGameState", (gameState) => {
        try {
            const user = getUser(socket.id);
            if (user) {
                // Add a timestamp to track latency
                const enhancedGameState = {
                    ...gameState,
                    _serverTimestamp: Date.now()
                };
                io.to(user.room).emit("updateGameState", enhancedGameState);
            }
        } catch (error) {
            logger.error(`Error updating game state for socket ${socket.id}:`, error);
            socket.emit("error", { message: "Failed to update game state" });
        }
    });

    socket.on("sendMessage", (payload, callback) => {
        const user = getUser(socket.id);
        io.to(user.room).emit("message", { user: user.name, text: payload.message });
        callback();
    });

    socket.on("quitRoom", () => {
        const user = removeUser(socket.id);
        if (user) io.to(user.room).emit("roomData", { room: user.room, users: getUsersInRoom(user.room) });
    });

    // Handle disconnection
    socket.on("disconnect", () => {
        activeConnections--;
        logger.info(`User ${socket.id} disconnected. Active connections: ${activeConnections}`);
        
        // Clean up user data on disconnect to prevent memory leaks
        const user = removeUser(socket.id);
        if (user) {
            io.to(user.room).emit("roomData", { 
                room: user.room, 
                users: getUsersInRoom(user.room) 
            });
            io.to(user.room).emit("userLeft", socket.id);
        }
    });
    
    // Handle socket errors
    socket.on("error", (error) => {
        logger.error(`Socket ${socket.id} error:`, error);
    });
});

// Global error handlers
process.on('uncaughtException', (error) => {
    logger.error('Uncaught Exception:', error);
    // Keep the process running despite the error
});

process.on('unhandledRejection', (reason, promise) => {
    logger.error('Unhandled Rejection at:', { promise, reason });
    // Keep the process running despite the rejection
});