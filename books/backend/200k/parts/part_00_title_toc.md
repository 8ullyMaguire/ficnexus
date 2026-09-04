# Building the FicHub Backend

## The Complete Guide to Building a Fanfiction Download Server in Rust

**200,000-Word Expanded Edition**

**By the FicHub Project**

*Version 2.0 — 2026*

---

> *"This book teaches you how to build a complete, production-ready backend server in Rust. From web scraping to EPUB generation, from Redis rate limiting to collaborative filtering recommendations — every chapter walks you through real code from a real project. This expanded 200,000-word edition includes extended code walkthroughs with line-by-line explanations, advanced Rust patterns, deep dives into async programming, database performance tuning, security hardening, monitoring, and a comprehensive troubleshooting guide."*

---

## About This Book

This is the expanded edition of the FicHub Backend book. The original edition covered 36 chapters across 8 parts, walking you through building a complete fanfiction download server in Rust. This 200,000-word edition expands every chapter with deeper explanations, more examples, real-world analogies, and practice exercises. It also adds seven entirely new parts covering advanced topics that will take your Rust backend skills to the next level.

Whether you're a beginner learning Rust for the first time or an experienced developer looking to deepen your understanding of async programming, database optimization, or security hardening, this book has something for you. Every code example comes from the actual FicHub source code — this isn't a toy project, it's a real, production-deployed application.

## Table of Contents

### Part 1: Welcome to Rust Backend Development
1. What We're Building
2. How the Web Works
3. Setting Up Your Workshop
4. Your First Rust Program

### Part 2: Axum Web Framework
5. Your First Web Server
6. Configuration and Error Handling
7. Connecting to PostgreSQL
8. Database Migrations and Models
9. CRUD Operations
10. The Axum Router and Middleware

### Part 3: Web Scraping
11. Understanding Fanfiction Sites
12. Building Web Scrapers
13. The Scraper Registry
14. AO3 Scraper Deep Dive
15. Other Site Scrapers

### Part 4: Export and Caching
16. Generating EPUB Files
17. HTML Bundles
18. The Disk Cache
19. Rate Limiting with Redis
20. The Export Flow

### Part 5: Recommendation Engine
21. Collaborative Filtering
22. The Collection Worker
23. Community Suggestions
24. Voting and Scoring

### Part 6: Server and Deployment
25. The Full Axum Router
26. Serving Static Files
27. Docker and Docker Compose
28. Cross-Compilation and Deploy

### Part 7: Advanced Backend Features
29. The Search System
30. The Tag System
31. The OPDS Catalog
32. The API Documentation
33. Performance and Optimization

### Part 8: Testing and Polish
34. Backend Testing
35. Production Hardening
36. What's Next?

### Part 9: Advanced Rust Patterns (NEW)
37. The Builder Pattern in Practice
38. Traits and Dynamic Dispatch
39. Smart Pointers and Interior Mutability
40. The Newtype Pattern
41. Error Handling Patterns Beyond `?`
42. Serde: The Serialization Powerhouse
43. Testing Patterns and Property-Based Testing

### Part 10: Async Deep Dive (NEW)
44. Understanding the Async Runtime
45. Futures, Tasks, and Executors
46. Tokio Internals
47. Async I/O Patterns
48. Structured Concurrency with JoinSet
49. Backpressure and Flow Control
50. Async Drop and Cleanup

### Part 11: Database Performance (NEW)
51. PostgreSQL Internals for Rust Developers
52. Connection Pool Tuning
53. Query Optimization with EXPLAIN
54. Indexing Strategies
55. Batch Operations and Bulk Inserts
56. Connection Pooling Deep Dive
57. Database Migrations at Scale

### Part 12: Security Hardening (NEW)
58. Input Validation and Sanitization
59. SQL Injection Prevention
60. Rate Limiting Security
61. CORS and Headers Security
62. Secrets Management
63. Dependency Auditing
64. Container Security

### Part 13: Monitoring and Observability (NEW)
65. Structured Logging with Tracing
66. Metrics Collection with Prometheus
67. Distributed Tracing
68. Health Checks and Readiness Probes
69. Alerting and On-Call

### Part 14: Troubleshooting Guide (NEW)
70. Common Build Errors and Fixes
71. Runtime Debugging Techniques
72. Memory Leak Investigation
73. Performance Profiling
74. Database Troubleshooting
75. Network and Connection Issues

### Part 15: Glossary (NEW)
76. Rust Terminology
77. Web Development Terms
77. Database Terminology
79. DevOps and Deployment Terms

---

## Preface to the Expanded Edition

When I first wrote the FicHub Backend book, I wanted to create the most comprehensive guide to building a real-world Rust backend application. The original 32,000-word edition covered the essentials — setting up the project, building scrapers, generating exports, implementing caching, and deploying with Docker. But readers kept asking for more: deeper explanations of async patterns, more detail on database optimization, real-world security practices, and troubleshooting guides for when things go wrong.

This 200,000-word expanded edition is the answer to those requests. Every original chapter has been expanded with:

- **Extended code walkthroughs** with line-by-line explanations of what each line does and why
- **Real-world analogies** that connect abstract concepts to everyday experiences
- **Practice exercises** at the end of each chapter that reinforce what you've learned
- **Common pitfalls** sections that highlight mistakes beginners make and how to avoid them
- **Performance tips** that show you how to write code that's not just correct but fast
- **Security considerations** that help you think about threats before they become problems

Beyond expanding existing chapters, I've added seven entirely new parts:

**Part 9: Advanced Rust Patterns** covers the builder pattern, traits and dynamic dispatch, smart pointers, the newtype pattern, advanced error handling, serde serialization, and property-based testing. These patterns are the building blocks of idiomatic Rust code.

**Part 10: Async Deep Dive** takes you inside Tokio's async runtime. You'll learn how futures work, how the executor schedules tasks, how to handle backpressure, and how to write efficient async code. This is the kind of knowledge that separates intermediate Rust developers from experts.

**Part 11: Database Performance** teaches you how to tune PostgreSQL for high-throughput web applications. You'll learn about connection pooling, query optimization, indexing strategies, and bulk operations. These skills are essential for any backend developer working with databases at scale.

**Part 12: Security Hardening** covers input validation, SQL injection prevention, rate limiting security, CORS configuration, secrets management, dependency auditing, and container security. Security isn't an afterthought — it's a fundamental part of building production software.

**Part 13: Monitoring and Observability** teaches you how to instrument your application with structured logging, metrics, distributed tracing, health checks, and alerting. You can't improve what you can't measure, and you can't fix what you can't see.

**Part 14: Troubleshooting Guide** is your go-to reference when things go wrong. It covers common build errors, runtime debugging, memory leak investigation, performance profiling, database troubleshooting, and network issues. Keep this section bookmarked — you'll need it.

**Part 15: Glossary** provides definitions for every technical term used in the book, organized by domain. If you encounter an unfamiliar term, look it up here.

### How to Read This Book

If you're new to Rust, start from the beginning and work your way through. Each chapter builds on the previous ones, and the concepts are introduced in a logical order.

If you already know Rust but want to learn about specific topics, use the table of contents to jump to the part that interests you. Each part is designed to be self-contained, though some knowledge from earlier parts may be helpful.

If you're a backend developer coming from Python, Node.js, or Go, pay special attention to Parts 1-4 (the basics) and Part 10 (async deep dive). The async model in Rust is fundamentally different from what you're used to, and understanding it will make you a much better developer.

### Prerequisites

This book assumes you have:
- Basic programming knowledge (variables, functions, loops, conditionals)
- Comfort with the command line
- A text editor (VS Code with rust-analyzer is recommended)
- A computer running Linux, macOS, or Windows with WSL2

You do NOT need to know Rust beforehand — we'll teach you everything you need as we go.

### A Note on the Code

Every code example in this book comes from the actual FicHub source code. We'll walk through real functions, real structs, and real SQL queries. When you see code, it's not a simplified toy — it's the actual implementation, explained line by line.

Some things to keep in mind:
- The code uses **Rust 2024 edition** features
- Dependencies are pinned to specific versions in `Cargo.toml`
- SQL queries use `sqlx`'s compile-time checking
- The async runtime is **Tokio**
- We use `tracing` for structured logging
- Error handling uses a custom `AppError` enum, not panics or unwrap

### Conventions Used in This Book

- **Code blocks** show Rust, SQL, shell commands, or configuration files
- **Bold text** highlights important concepts or new terms
- **Blockquotes** contain tips, warnings, or real-world analogies
- **🧪 Try It Yourself** sections invite you to experiment
- **⚠️ Watch Out** sections warn about common pitfalls
- **📝 Practice Exercise** sections at chapter ends reinforce learning
- **Real-world analogy** boxes connect abstract concepts to everyday experiences

### Acknowledgments

This book wouldn't exist without the open-source community. Thanks to the Axum, Tokio, SQLx, and serde teams for building the amazing tools we use every day. Thanks to the Archive of Our Own volunteers for running such a wonderful platform. And thanks to every reader who asked questions, reported issues, and suggested improvements.

Let's build something amazing together.

---

