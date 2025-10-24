# Zunno - Blockchain's First Verifiably Fair UNO Card Game

A digital edition of the classic **UNO game**, designed with **fairness** and **verifiability** at its core.  
Gameplay is transparent, secure, and trustless—powered by **smart contracts**, **zero-knowledge proofs**, and **verifiable randomness**.

Zunno demonstrates the evolution from decentralized gaming to cryptographically provable fairness.

---

## 🎯 Development Roadmap

### **Phase One (β.v.01): Decentralized On-Chain Gameplay**

The first phase establishes the foundation of decentralized UNO gameplay:

- **Client-Side Application**  
  React-based frontend enabling players to interact directly with blockchain smart contracts
  
- **On-Chain Game State Management**  
  All game sessions, player participation, hands, moves, and results are recorded on-chain for transparency
  
- **Smart Contract on Base Sepolia**  
  Solidity contracts manage the global and historical game state, ensuring no centralized authority controls gameplay
  
- **Server-Assisted Card Distribution**  
  Initial randomness and card shuffling handled by Rust backend services using cryptographically secure random number generation

**Status:** ✅ **Complete** - Deployed and operational on Base Sepolia testnet

---

### **Phase Two (β.v.02): Verifiable Fairness and ZK Integration**

The second phase introduces cryptographic guarantees of fairness:

- **Verifiable Randomness via Chainlink VRF**  
  Integration of Chainlink's Verifiable Random Function for provably fair card shuffling, eliminating predictability and manipulation
  
- **Zero-Knowledge Proof Generation**  
  Implementation of Succinct's SP1 prover to generate cryptographic proofs attesting to fair shuffling and dealing using the Fisher-Yates algorithm
  
- **On-Chain Proof Verification with ZKVerify**  
  ZKVerify integration validates ZK proofs on-chain, providing transparent auditability without revealing the shuffle seed
  
- **Enhanced Rust Backend Orchestration**  
  Backend coordinates proof generation, VRF requests, and gameplay logic alongside smart contract interactions
  
- **End-to-End Verifiable Fairness**  
  Complete pipeline from randomness generation → proof creation → on-chain verification → gameplay execution

**Status:** 🚧 **In Progress** - Backend logic implementation completed; proceeding to testing next.

---

## ✨ Key Features

- **Trustless Gameplay**  
  No centralized server can manipulate outcomes—game logic enforced by smart contracts
  
- **Cryptographic Fairness**  
  Fisher-Yates shuffling with Chainlink VRF randomness and SP1 zero-knowledge proofs
  
- **Transparent Verification**  
  Anyone can verify game fairness through on-chain ZKVerify proof validation
  
- **React Frontend**  
  Intuitive interface for seamless blockchain interaction and real-time gameplay
  
- **Rust Backend**  
  High-performance backend handling proof generation, VRF coordination, and API endpoints

---

## 🏗️ Technical Architecture

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│                 │◄────────┤                  │◄────────┤                 │
│  React Client   │         │  Rust Backend    │         │  Smart Contract │
│   (Frontend)    │────────►│   (API + ZK)     │         │  (Base-Sepolia) │
│                 │         │                  │         │                 │
└────────┬────────┘         └────────┬─────────┘         └────────┬────────┘
         │                           │                            │
         │                           │                            │
         └───────────────────────────┼────────────────────────────┘
                                     │                            │
                                     │                            ▼
                                     │                   ┌─────────────────┐
                                     │                   │                 │
                                     │                   │  Chainlink VRF  │
                                     │                   │  (Randomness)   │
                                     │                   │                 │
                                     │                   └─────────────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │                  │
                            │  SP1 Prover      │
                            │  (ZK Proofs)     │
                            │                  │
                            └────────┬─────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │                  │
                            │    ZKVerify      │
                            │  (Verification)  │
                            │                  │
                            └──────────────────┘
```

### **Module Breakdown**

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Frontend** | React | Player interface and wallet integration |
| **Smart Contracts** | Solidity | On-chain state management and game rules |
| **Backend** | Rust | API endpoints, proof orchestration, VRF coordination |
| **Randomness** | Chainlink VRF | Verifiable random seed generation |
| **Proof System** | Succinct SP1 | Zero-knowledge proof generation for shuffles |
| **Verification** | ZKVerify | On-chain proof validation |

---

## 📂 Project Structure

```
/client      → React application (UI/UX)
/server      → Rust backend (shuffling, proofs, API endpoints)
/contracts   → Solidity smart contracts (Base Sepolia/Mainnet)
```

---

## ⚖️ Verifiable Fairness

**Phase One:** Server-generated randomness using cryptographically secure RNGs

**Phase Two:** Chainlink VRF provides verifiable on-chain randomness, which seeds the Fisher-Yates shuffle algorithm. The shuffle execution is proven via Succinct's SP1, and the resulting zero-knowledge proof is verified by ZKVerify—creating an unbreakable chain of cryptographic guarantees.

---

## 🪄 Live App

Try the live game Zunno β.v.01 [here](https://zunno.xyz) . 
We are also live on BaseApp and [Farcaster](https://farcaster.xyz/miniapps/sT0wxMVbxIg_/zunno)

---

## 🚀 Future Enhancements

- 🎨 **Enhanced UI/UX**: Animations, player dashboards, game replays, and leaderboards
- 🌐 **Mainnet Deployment**: Transition from Base Sepolia to Base Mainnet for production
- 🏆 **Tournament Mode**: Multi-round competitions with prize pools
- 🔄 **Cross-Chain Support**: Enable gameplay across multiple blockchain networks
- 📊 **Analytics Dashboard**: On-chain game statistics and fairness auditing tools

---

## 🔮 Vision

Zunno demonstrates how blockchain technology can eliminate trust requirements in digital card games. By combining smart contracts, verifiable randomness, and zero-knowledge proofs, we create provably fair entertainment where every shuffle, deal, and outcome is cryptographically guaranteed and publicly auditable.

The future of digital gaming is transparent, fair, and trustless.

---

## 🤝 Contributing

Contributions, suggestions, and feedback are welcome!
