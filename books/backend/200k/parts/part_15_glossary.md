# Part 15: Glossary

---

# Chapter 76: Rust Terminology

## A

**Abstraction** — A way of hiding implementation details while exposing a clean interface. Rust uses traits, generics, and modules for abstraction. Zero-cost abstractions provide high-level APIs without runtime overhead.

**Arc (Atomic Reference Counted)** — A thread-safe reference-counted pointer. Arc allows multiple owners of the same data across threads. When the last Arc is dropped, the data is freed. Used extensively in async Rust for sharing state.

**Async/Await** — Rust's syntax for asynchronous programming. An async function returns a Future instead of a value directly. The .await keyword suspends execution until the Future completes.

**Attribute** — A declaration that applies to a Rust item (function, struct, enum, etc.). Attributes start with # and can be simple (#[test]) or take parameters (#[derive(Debug)]).

## B

**Borrow Checker** — Rust's compile-time system that tracks references to ensure memory safety. It ensures that references are always valid, that you can't have both mutable and immutable references simultaneously, and that references don't outlive the data they point to.

**Box** — A smart pointer that allocates data on the heap. Box<T> provides ownership semantics for heap-allocated data. Used for recursive types, trait objects, and when you need heap allocation.

**Builder Pattern** — A creational design pattern that constructs complex objects step by step. In Rust, builders take ownership of self in each method, enabling method chaining.

## C

**Cargo** — Rust's build system and package manager. Cargo handles dependency management, building, testing, benchmarking, and documentation generation.

**Crate** — A Rust library or binary. Crates are the unit of compilation and distribution. They're published to crates.io.

**Closure** — An anonymous function that can capture variables from its enclosing scope. Closures use |params| syntax and can be Fn, FnMut, or FnOnce.

**Compile-Time Checking** — Rust's ability to verify correctness at compile time. SQLx uses this to check SQL queries against the actual database schema.

**Concurrency** — Multiple tasks making progress during overlapping time periods. Rust ensures concurrency safety through ownership and Send/Sync traits.

**Connection Pool** — A cache of database connections that are reused across requests. SQLx manages connection pools automatically.

## D

**Deadlock** — A situation where two or more tasks are waiting for each other to release resources, preventing any of them from progressing. Rust's ownership system prevents some deadlocks, but not all.

**Dependency Injection** — A pattern where dependencies are passed into a component rather than created internally. FicHub uses Axum's State extractor for dependency injection.

**Dispatch** — The mechanism for calling a method on a trait object. Static dispatch resolves at compile time (generics), while dynamic dispatch resolves at runtime (dyn Trait).

**Drop** — The cleanup code that runs when a value goes out of scope. Drop is an automatic mechanism for resource cleanup.

## E

**Enum** — A type that can be one of several variants. Enums in Rust can carry data in each variant, making them more powerful than enums in most languages.

**Error Propagation** — The mechanism for passing errors up the call stack. Rust uses the ? operator for automatic error propagation.

**Executor** — The component that polls futures to make progress. Tokio's executor schedules tasks across worker threads.

**Expression** — A piece of code that evaluates to a value. In Rust, almost everything is an expression, including if statements, match expressions, and blocks.

## F

**Feature Flag** — A compile-time option that enables or disables code. Cargo.toml defines feature flags that control which dependencies are compiled.

**Future** — A value that represents a computation that will complete in the future. Futures are lazy — they don't do anything until polled.

## G

**Generic** — A parameterized type that works with multiple concrete types. Generics provide code reuse without sacrificing type safety.

## H

**Handle** — A reference to a resource that can be used to interact with it. Tokio handles provide access to the runtime for spawning tasks.

## I

**Immutable** — Unable to be changed after creation. Rust variables are immutable by default.

**Import** — Bringing a name into scope. Rust uses the use keyword for imports.

**Iterator** — A type that produces a sequence of values. Iterators enable functional-style data processing with methods like map, filter, and fold.

## J

**JoinSet** — Tokio's structured concurrency primitive. JoinSet manages a collection of spawned tasks and ensures they all complete.

## K

**Key-Value Store** — A data storage paradigm where data is stored as key-value pairs. Redis is a key-value store used for caching and rate limiting.

## L

**Lifetime** — The scope during which a reference is valid. Lifetimes prevent dangling references by ensuring references don't outlive the data they point to.

**Linting** — Static analysis of code to find potential issues. Clippy is Rust's linting tool.

## M

**Macro** — Code that generates other code at compile time. Macros use the macro_rules! syntax or procedural macro crates.

**Middleware** — Code that wraps request handling to add functionality. Axum uses Tower middleware for CORS, compression, tracing, etc.

**Move** — Transferring ownership of a value from one variable to another. Move semantics are fundamental to Rust's ownership model.

**Mutex** — A mutual exclusion lock that allows only one thread to access data at a time. Mutex provides interior mutability with synchronization.

## N

**Newtype** — A pattern that wraps an existing type in a single-field tuple struct. Newtypes provide type safety and allow implementing traits on the wrapper.

## O

**Ownership** — Rust's memory management system where each value has exactly one owner. When the owner goes out of scope, the value is dropped.

## P

**Pattern Matching** — A mechanism for destructuring values and branching based on their structure. Match expressions in Rust are exhaustive — you must handle every case.

**Pin** — A wrapper that prevents a value from being moved in memory. Pin is essential for self-referential structs, which are common in async code.

**Pool** — A reusable collection of resources. Connection pools manage database connections; thread pools manage worker threads.

**Profiling** — Measuring where a program spends its time and resources. Profiling helps identify performance bottlenecks.

## Q

**Query Builder** — A type-safe way to construct SQL queries. SQLx's QueryBuilder ensures all values are properly parameterized.

## R

**RAII (Resource Acquisition Is Initialization)** — A pattern where resources are acquired in constructors and released in destructors. Rust's ownership system implements RAII automatically.

**Rate Limiting** — Controlling how many requests a client can make in a given time period. Token bucket algorithms implement rate limiting.

**Reference** — A pointer to data that doesn't take ownership. References allow borrowing data without transferring ownership.

**Registry** — A pattern for managing a collection of implementations. FicHub's ScraperRegistry maps URLs to the appropriate scraper.

## S

**Scalar Type** — A type that represents a single value. Rust's scalar types are integers, floating-point numbers, booleans, and characters.

**Scrutinee** — The value being matched in a match expression.

**Semantics** — The meaning of code. Move semantics, copy semantics, and reference semantics describe how values are transferred.

**Send** — A marker trait indicating a type can be safely sent between threads. Most types in Rust are Send.

**Serde** — The serialization framework for Rust. Serde handles converting between Rust types and formats like JSON, TOML, and YAML.

**Slice** — A reference to a contiguous sequence of elements. Slices provide a view into an array or vector.

**Smart Pointer** — A pointer that provides additional functionality beyond raw pointers. Box, Rc, Arc, and RefCell are smart pointers.

**State Machine** — A model of computation where the system can be in one of a finite number of states. Async functions are compiled into state machines.

**Struct** — A type with named fields. Structs group related data together.

**Sync** — A marker trait indicating a type can be safely shared between threads. Immutable references are typically Sync.

## T

**Thread** — A sequence of instructions that can be managed independently. Rust threads are OS threads; async tasks are lightweight green threads.

**Tokio** — The async runtime for Rust. Tokio provides the event loop, task scheduler, timers, and I/O primitives.

**Trait** — A collection of methods that define shared behavior. Traits are similar to interfaces in other languages.

**Trait Object** — A value whose type is determined at runtime. Trait objects enable dynamic dispatch and heterogeneous collections.

**Type Inference** — Rust's ability to determine types automatically. You often don't need to specify types explicitly.

## U

**Unsafe** — A keyword that allows operations that the compiler can't verify for safety. Unsafe code is used sparingly and must be manually audited for correctness.

## V

**Variable** — A name bound to a value. Variables can be immutable (default) or mutable (with mut).

**Variant** — One of the possible states of an enum. Each variant can carry different data.

## W

**Waker** — A handle that notifies the executor when a task can make progress. Wakers are essential for async I/O.

**Worker Thread** — A thread that executes tasks from a queue. Tokio's worker threads process async tasks.

---

# Chapter 77: Web Development Terms

## A

**API (Application Programming Interface)** — A set of rules for how software components interact. FicHub's API defines how clients can request story exports and metadata.

**Atom** — A web feed format similar to RSS. OPDS feeds use Atom as the base format.

**Authentication** — Verifying the identity of a user or client. FicHub uses API keys for curator authentication.

**Authorization** — Determining what an authenticated user is allowed to do.

## B

**Backend** — The server-side of a web application. FicHub's backend handles scraping, caching, and API responses.

**Body** — The payload of an HTTP request or response. Bodies can contain JSON, HTML, files, or other data.

## C

**Cache** — A temporary storage layer that speeds up data retrieval. FicHub uses disk and database caching.

**CORS (Cross-Origin Resource Sharing)** — A mechanism that allows web pages to request resources from different origins.

**CRUD** — Create, Read, Update, Delete — the four basic database operations.

## D

**DNS (Domain Name System)** — Translates domain names to IP addresses.

## H

**Handler** — A function that processes an HTTP request and returns a response.

**Header** — Metadata in an HTTP request or response. Headers include Content-Type, Authorization, etc.

**HTTP (HyperText Transfer Protocol)** — The protocol for transferring web pages and API data.

**HTTPS** — HTTP over TLS/SSL, providing encrypted communication.

## J

**JSON (JavaScript Object Notation)** — A lightweight data format. FicHub's API returns JSON responses.

## M

**Middleware** — Code that processes requests before or after the handler. Axum uses Tower middleware.

## O

**OPDS (Open Publication Distribution System)** — A standard for e-book catalogs. FicHub publishes an OPDS catalog.

## P

**Payload** — The data sent in an HTTP request or response body.

## Q

**Query Parameter** — Key-value pairs in a URL's query string. FicHub uses query parameters for search and export requests.

## R

**Rate Limiting** — Controlling request frequency. FicHub limits requests to protect upstream sites.

**REST (Representational State Transfer)** — An architectural style for web APIs.

**Router** — The component that maps URLs to handlers. Axum's Router dispatches requests to the appropriate handler.

## S

**SPA (Single-Page Application)** — A web app that loads a single page and updates content dynamically. FicHub's frontend is a SvelteKit SPA.

**Status Code** — A three-digit number indicating the result of an HTTP request (200 OK, 404 Not Found, etc.).

## T

**TLS (Transport Layer Security)** — The protocol that encrypts HTTPS connections.

## U

**URL (Uniform Resource Locator)** — A web address that specifies the location of a resource.

## V

**Versioning** — API version management. FicHub uses /api/v0/ for versioned endpoints.

---

# Chapter 78: Database Terms

## A

**ACID** — Atomicity, Consistency, Isolation, Durability — properties that guarantee database transactions are processed reliably.

**Aggregate Function** — A function that combines multiple rows (COUNT, SUM, AVG, etc.).

**ALTER TABLE** — SQL command to modify a table's structure.

**Index** — A data structure that speeds up data retrieval. Indexes allow PostgreSQL to find rows without scanning the entire table.

**INSERT** — SQL command to add new rows to a table.

## B

**Batch** — A group of operations executed together. Batch inserts are faster than individual inserts.

**B-Tree** — The default index type in PostgreSQL. B-trees are efficient for equality and range queries.

## C

**Column** — A vertical field in a table that stores a specific type of data.

**Connection Pool** — A cache of database connections reused across requests.

**CREATE TABLE** — SQL command to create a new table.

**Cursor** — A pointer to a result set. Cursors allow processing results row by row.

## D

**DDL (Data Definition Language)** — SQL commands that define database structure (CREATE, ALTER, DROP).

**DML (Data Manipulation Language)** — SQL commands that manipulate data (INSERT, UPDATE, DELETE).

## E

**EXPLAIN** — SQL command that shows the query execution plan.

## F

**Foreign Key** — A column that references a primary key in another table. Foreign keys enforce referential integrity.

## G

**GIN (Generalized Inverted Index)** — An index type in PostgreSQL good for full-text search and arrays.

## I

**INNER JOIN** — Combines rows from two tables where the join condition is met.

**Isolation Level** — The degree to which transactions are isolated from each other.

## J

**JOIN** — SQL operation that combines rows from multiple tables.

## L

**LEFT JOIN** — Combines rows from two tables, including all rows from the left table.

**Lock** — A mechanism to prevent concurrent access to data.

## M

**Migration** — A versioned SQL script that modifies the database schema.

## N

**NULL** — A special value indicating missing data.

## O

**ORDER BY** — SQL clause that sorts query results.

## P

**Primary Key** — A column (or combination) that uniquely identifies each row.

**Query Planner** — PostgreSQL's component that determines the most efficient query execution plan.

## R

**Referential Integrity** — The property that foreign keys always point to valid primary keys.

**Row** — A horizontal record in a table.

**ROLLBACK** — Reverting a transaction to its state before it began.

## S

**Schema** — The structure of a database (tables, columns, indexes, constraints).

**SELECT** — SQL command to query data from tables.

**Sequence** — An auto-incrementing number generator in PostgreSQL.

**SQL (Structured Query Language)** — The language for managing relational databases.

**SQLx** — A Rust library for SQL databases with compile-time query checking.

## T

**Table** — A collection of related data organized in rows and columns.

**Transaction** — A group of SQL operations that are treated as a single unit.

**Trigger** — A stored procedure that runs automatically when a specific event occurs.

## U

**UPDATE** — SQL command to modify existing rows.

**UPSERT** — INSERT with ON CONFLICT DO UPDATE — insert or update in one operation.

## V

**VIEW** — A virtual table based on a query result.

## W

**WHERE** — SQL clause that filters rows based on conditions.

---

# Chapter 79: DevOps and Deployment Terms

## A

**Alpine Linux** — A minimal Linux distribution used as a base for Docker images.

**Ansible** — An automation tool for configuration management and deployment.

## B

**Build Stage** — The first phase of a multi-stage Docker build where the application is compiled.

## C

**CDN (Content Delivery Network)** — A distributed network of servers that delivers content based on user location.

**CI/CD (Continuous Integration/Continuous Deployment)** — Automated testing and deployment pipelines.

**Container** — A lightweight, isolated environment for running applications. Docker containers package applications with their dependencies.

**Container Registry** — A storage system for Docker images (Docker Hub, GitHub Container Registry).

## D

**Docker** — A platform for building and running containerized applications.

**Docker Compose** — A tool for defining and running multi-container Docker applications.

**Dockerfile** — A script that defines how to build a Docker image.

## E

**Environment Variable** — A key-value pair in the shell that configures applications.

## G

**Git** — A version control system for tracking code changes.

## H

**Health Check** — A test that verifies a service is running correctly.

## I

**Image** — A read-only template for creating Docker containers.

## J

**JSON** — A data format used for configuration and API communication.

## K

**Kubernetes** — A container orchestration platform for managing containerized applications.

## L

**Load Balancer** — A device that distributes network traffic across multiple servers.

## M

**Manifest** — A file that defines container configuration (docker-compose.yml, Kubernetes manifests).

## N

**Nginx** — A web server and reverse proxy.

## P

**Payload** — The data sent in an HTTP request or response.

**Port** — A network endpoint number. Containers map internal ports to host ports.

**PostgreSQL** — An open-source relational database.

## R

**Redis** — An in-memory key-value store used for caching and rate limiting.

**Reverse Proxy** — A server that forwards client requests to backend servers.

**Rollback** — Reverting to a previous version after a failed deployment.

## S

**Scalability** — The ability to handle increased load by adding resources.

**Secret** — Sensitive configuration data (passwords, API keys) that must be protected.

**Service** — A long-running process that handles requests. In Docker Compose, services are defined in docker-compose.yml.

## T

**TLS (Transport Layer Security)** — The protocol that encrypts HTTPS connections.

## V

**Volume** — Persistent data storage in Docker. Volumes survive container restarts.

## W

**Worker Thread** — A thread that processes tasks from a queue. Tokio uses worker threads for async task execution.

