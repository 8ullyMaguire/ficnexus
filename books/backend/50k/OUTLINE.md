# Building the FicHub Backend (50k)

> 50078 words

## Table of Contents

# Building the FicHub Backend
## A Complete Guide to Building a Fanfiction Download Server in Rust
### What You'll Learn
### Who Is This Book For?
### The FicHub Source Code
# Chapter 1: Welcome to FicHub — The Big Picture
## What Is FicHub?
## Why Rust?
## The Journey Ahead
# Chapter 2: Hello, Rust! — Your First Steps
## What Is a Programming Language?
## Your Very First Rust Program
## Understanding Compilation
## Variables and Types
### Types of Variables
### String vs &str
### Mutability
## Functions
### Naming Conventions
### The FicHub Way
## Enums and Pattern Matching
### Enums with Data
### Why Enums Are Great
## Structs: Grouping Related Data
### The `pub` Keyword
## Error Handling with Result
## Loops and Iterators
# Chapter 3: Cargo & Dependencies — Your Tool Belt
## What Is Cargo?
## The Cargo.toml File
# Web framework & runtime
# Database
# Redis
# HTTP client
# HTML parsing
# EPUB generation
# Template engine
# Serialization
# Config
# Logging
# Metrics
# UUIDs
# Hashing
# Date/time
# Compression (zip)
# Async traits
# Path handling
# Hashing/encoding
# Testing
### The [package] Section
### The [dependencies] Section
## Our Key Dependencies
### Axum — Our Web Framework
### Tokio — Our Async Runtime
### SQLx — Our Database Driver
### Redis — Our Caching Helper
### Reqwest — Our HTTP Client
### Scraper — Our HTML Parser
### EPUB Builder — Our Book Maker
### Serde — Our Serialization Library
#[derive(Serialize, Deserialize)]
### Tracing — Our Logging System
## Adding a New Dependency
## Dev Dependencies — Tools for Testing
## Understanding Features
## The Cargo.lock File
## Cargo Commands Cheat Sheet
# Chapter 4: Configuration & main.rs — Setting Up Shop
## The Entry Point: main.rs
#[tokio::main]
### The `pub mod` Lines
### The `#[tokio::main]` Attribute
#[tokio::main]
### Loading Environment Variables
### Setting Up Logging
### Loading Configuration
## The Config Struct
### The `from_env` Method
### The .env File
## The lib.rs File
