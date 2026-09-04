# Building the FicHub Backend (200k)

> 200037 words

## Table of Contents

# Building the FicHub Backend
## A Complete Guide to Building a Fanfiction Backend in Rust
# Table of Contents
## Part I: Welcome to Backend Development!
## Part II: The Foundation — Configuration & Startup
## Part III: The Web Server — Axum in Action
## Part IV: The Database — PostgreSQL & SQLx
## Part V: Scraping the Web — The Scraper System
## Part VI: Export & Download — Creating EPUBs and More
## Part VII: Rate Limiting & Security
## Part VIII: The Tagging System
## Part IX: The Recommendation Engine
## Part X: Advanced Search
## Part XI: OPDS Catalog
## Part XII: Putting It All Together
## Advanced Topics
# Preface
## How to Read This Book
# Part I: Welcome to Backend Development!
# Chapter 1: What is FicHub?
## 1.1 The Big Picture
## 1.2 What is a Backend?
## 1.3 What is Rust?
## 1.4 What We Will Build
## 1.5 A Map of Our Journey
## Practice Exercises
# Chapter 2: Meet the Technologies
## 2.1 The Technology Stack
### Rust (The Programming Language)
#[derive(Serialize, Deserialize)]
### Axum (The Web Framework)
### PostgreSQL (The Database)
### SQLx (The Database Library)
### Redis (The Cache)
### reqwest (The HTTP Client)
### scraper (The HTML Parser)
### epub-builder (The EPUB Generator)
### tower-http (The Middleware Layer)
## 2.2 How These Technologies Connect
## 2.3 Key Concepts to Remember
## 2.4 Fun Facts
## Practice Exercises
# Chapter 3: Setting Up Your Development Environment
## 3.1 What You Need
## 3.2 Installing Rust
# Should print something like: rustc 1.75.0 (82e1608df 2023-12-21)
# Should print something like: cargo 1.75.0 (1d8b05fdd 2023-11-20)
## 3.3 Installing PostgreSQL
## 3.4 Installing Redis
# Should print: PONG
## 3.5 Cloning the Repository
## 3.6 Setting Up Environment Variables
# Database connection
# Redis connection
# Where to store exported files
# Temporary files directory
# Port to listen on
# Logging level (info for normal, debug for verbose)
## 3.7 Building the Project
## 3.8 Running the Server
## 3.9 Project Structure Overview
## Practice Exercises
# Chapter 4: Understanding the Project Structure
## 4.1 Module Organization
## 4.2 The Dependency Graph
## 4.3 The Public API
## 4.4 Why This Structure?
## Practice Exercises
# Chapter 5: Your First Look at the Code
## 5.1 The Entry Point: main.rs
#[tokio::main]
### `#[tokio::main]`
### `dotenvy::dotenv().ok()`
### `tracing_subscriber::fmt()...init()`
### `config::Config::from_env()`
### `server::run(config).await`
## 5.2 The Server Setup
## 5.3 The run() Function
## 5.4 Summary
## Practice Exercises
