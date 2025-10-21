// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./SubscriptionConsumer.sol";

contract UnoGame is ReentrancyGuard {

    SubscriptionConsumer public randomWordContract;

    constructor(address _randomWordContractAddress) {
        randomWordContract = SubscriptionConsumer(_randomWordContractAddress);
    }

    uint256 private _gameIdCounter;
    uint256[] private _activeGames;

    enum GameStatus { NotStarted, Started, Ended }

    struct Game {
        uint256 id; 
        address[] players; 
        GameStatus status; 
        uint256 startTime; 
        uint256 endTime; 
        bytes32 gameHash; 
        bytes32[] moves;
    }

    mapping(uint256 => Game) private games;

    event GameCreated(uint256 indexed gameId, address creator);
    event PlayerJoined(uint256 indexed gameId, address player);
    event GameStarted(uint256 indexed gameId);
    event MoveCommitted(uint256 indexed gameId, bytes32 moveHash);
    event GameEnded(uint256 indexed gameId);
    event RequestFulfilled(
        uint256 indexed requestId,
        uint256 randomWord
    ); 

    modifier validateGame(uint256 _gameId, GameStatus requiredStatus) {
        require(_gameId > 0 && _gameId <= _gameIdCounter, "Invalid game ID");

        Game storage game = games[_gameId];
        require(game.status == requiredStatus, "Game is not in the required status");
        _;
    }

    function requestRandomWords() public returns(uint256) {
        uint256 requestId = randomWordContract.requestRandomWords(false);

        return requestId;
    }

    function getRandomWords (uint256 req_Id) public view returns (uint256){
        uint256 _lastRequestId = randomWordContract.lastRequestId();

        (bool fulfilled, uint256[] memory randomWords) = randomWordContract.getRequestStatus(req_Id);

        require(fulfilled, "Request not fulfilled yet");
        emit RequestFulfilled(req_Id, randomWords[0]);
        return randomWords[0]; 
    }

    function createGame(address _creator) external nonReentrant returns (uint256) {
        _gameIdCounter++;
        uint256 newGameId = _gameIdCounter;

        games[newGameId] = Game({
            id: newGameId,
            players: new address[](0),
            status: GameStatus.NotStarted,
            startTime: block.timestamp,
            endTime: 0, 
            gameHash: "",
            moves: new bytes32[](0)
        });
        _activeGames.push(newGameId);
        emit GameCreated(newGameId, _creator);
        return newGameId;
    }

    function startGame(uint256 gameId) external validateGame(gameId, GameStatus.NotStarted) {
        Game storage game = games[gameId];
        require(game.players.length >= 2, "Not enough players");
        
        game.status = GameStatus.Started;
        
        emit GameStarted(gameId);
    }

    function joinGame(uint256 gameId, address _joinee) external nonReentrant validateGame(gameId, GameStatus.NotStarted){
        Game storage game = games[gameId];
        require(game.players.length < 10, "Game is full");

        game.players.push(_joinee);
        emit PlayerJoined(gameId, _joinee);
    }

    function commitMove(uint256 gameId, bytes32 moveHash) external validateGame(gameId, GameStatus.Started) {
        Game storage game = games[gameId];
        game.moves.push(moveHash);
        emit MoveCommitted(gameId, moveHash);
    }

    function endGame(uint256 gameId, bytes32 gameHash) external validateGame(gameId, GameStatus.Started){
        Game storage game = games[gameId];

        game.status = GameStatus.Ended;
        game.gameHash = gameHash;
        removeFromActiveGames(gameId);
        emit GameEnded(gameId);
    }

    function removeFromActiveGames(uint256 gameId) internal {
        for (uint256 i = 0; i < _activeGames.length; i++) {
            if (_activeGames[i] == gameId) {
                _activeGames[i] = _activeGames[_activeGames.length - 1];
                _activeGames.pop();
                break;
            }
        }
    }

    function getActiveGames() external view returns (uint256[] memory) {
        return _activeGames;
    }

    function getNotStartedGames() external view returns (uint256[] memory) {
        uint256[] memory notStartedGames = new uint256[](_activeGames.length);
        uint256 count = 0;

        for (uint256 i = 0; i < _activeGames.length; i++) {
            uint256 gameId = _activeGames[i];
            if (games[gameId].status == GameStatus.NotStarted) {
                notStartedGames[count] = gameId;
                count++;
            }
        }

        // Resize the array to fit the actual number of not started games
        uint256[] memory result = new uint256[](count);
        for (uint256 j = 0; j < count; j++) {
            result[j] = notStartedGames[j];
        }

        return result;
    }

    function getGame(uint256 gameId) public view returns (
        uint256 id,
        address[] memory players,
        GameStatus status,
        uint256 startTime,
        uint256 endTime,
        bytes32 gameHash,
        bytes32[] memory moves
    ) {
        Game storage game = games[gameId];
        return (
            game.id,
            game.players,
            game.status,
            game.startTime,
            game.endTime,
            game.gameHash,
            game.moves
        );
    }
}
