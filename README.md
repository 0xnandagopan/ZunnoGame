# Zunno - Blockchain's 1st Uno card game on zkVerify built as part of zkmonk zkforge bootcamp hackathon.

A digital edition of the classic **UNO game**, designed with **fairness** and **verifiability** at its core.  
Gameplay is transparent, secure, and trustless—powered by **smart contracts**, **Rust backend services**, and upcoming **zero-knowledge proofs**.

---

## ✨ Features

- **Fair & Verifiable Gameplay**  
  All critical aspects—game sessions, player participation, initial hands, moves, and results—are tracked **on-chain**.  
- **React Frontend**  
  A client-side interface for seamless play and interaction with the blockchain.  
- **Smart Contract on Base**  
  Securely manages the global and historical game state.  
- **Rust Backend**  
  Ensures fair shuffling and dealing of cards, exposing APIs for player actions.  

---

## 🛠️ Current Setup

- ✅ **Smart Contract** deployed on **Base Sepolia** for managing historical game state  
- ✅ **Rust Backend** exposing essential endpoints for user-facing game functions  

---

## 🚀 Roadmap

- 🔒 **ZK-Based Proof Generation**  
  Implement provably fair shuffling and dealing using **Succinct’s SP1 prover**  
- ✅ **On-Chain Proof Verification**  
  Integrate with **ZkVerify** for validating proofs on-chain  
- 🎨 **Enhanced Frontend**  
  Expand gameplay UI/UX with animations, player dashboards, and replays  
- 🌐 **Mainnet Deployment**  
  Transition from Base Sepolia to **Base Mainnet** for production readiness  

---

## 📂 Project Structure

/client     → React app (UI) 
/server      → Rust backend (shuffling, dealing, endpoints) 
/contracts    → Solidity smart contracts (Base Sepolia)

---
## ⚖️ Verifiable Fairness

During gameplay, an ordered deck of UNO cards is shuffled using the **Fisher–Yates algorithm** and distributed among players, with the remaining shuffled cards forming the draw pile.

The shuffling process incorporates randomness via **Chainlink VRF**, removing any need for blind trust or reliance on predictable shuffle arrangements.

🔗 [ZkVerify verification result for randomized shuffle implementation](https://zkverify-testnet.subscan.io/extrinsic/0x8baaa3526e3615c4a9625b31e6ff574e5886a376dfdb196d727640bd1f9a5b0c)

---

## 🎥 Demo

Watch the demo here: [Zunno Demo Video](https://drive.google.com/drive/folders/1kOLVC7rXofUjcnWJIJguSVhH-AgqAj9N)

---

## 🔮 Vision

The goal is to make digital card games provably fair—eliminating hidden logic, ensuring verifiable randomness, and building trustless entertainment on-chain.

