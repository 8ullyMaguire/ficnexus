# Part 10: Async Deep Dive

---

# Chapter 44: Understanding the Async Runtime

Rust's async/await syntax is syntactic sugar for state machines. Understanding how it works under the hood is crucial for writing efficient async code.

## Futures and Executors

A future is a value that represents a computation that will complete in the future:

```rust
// A future is any type implementing the Future trait
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MyFuture {
    data: String,
}

impl Future for MyFuture {
    type Output = String;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the computation is complete
        // If yes, return Poll::Ready(result)
        // If no, return Poll::Pending and wake when ready
        Poll::Ready(self.data.clone())
    }
}
```

**Real-world analogy:** A future is like a restaurant order ticket. You place an order (create the future), and the kitchen (executor) works on it. When it's ready, you get your food (the result). While waiting, you can do other things (other tasks can run).

## The Tokio Runtime

Tokio is the async runtime that executes futures:

```rust
#[tokio::main]
async fn main() {
    // Tokio creates a multi-threaded runtime
    // and runs the async main function
    
    let result = fetch_data().await;
    println!("Got: {}", result);
}
```

The `#[tokio::main]` attribute transforms your async main into:

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = fetch_data().await;
        println!("Got: {}", result);
    });
}
```

### Runtime Configuration

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)           // Number of worker threads
    .enable_all()                // Enable all features
    .thread_name("fichub-worker") // Thread name prefix
    .on_thread_start(|| {
        tracing::debug!("Worker thread started");
    })
    .build()
    .unwrap();
```

## Task Scheduling

Tokio uses a work-stealing scheduler. Each worker thread has a local task queue. When a thread runs out of tasks, it "steals" tasks from other threads' queues.

```rust
// Spawn a new task
tokio::spawn(async {
    // This runs concurrently with other tasks
    fetch_data().await;
});

// Spawn with a specific runtime handle
let handle = tokio::runtime::Handle::current();
handle.spawn(async {
    background_work().await;
});
```

## 📝 Practice Exercises

1. **Future Implementation:** Implement the `Future` trait for a custom type that returns a value after a delay.

2. **Task Spawning:** Spawn 10 tasks that each print a number, and verify they run concurrently.

3. **Runtime Configuration:** Create a runtime with 2 worker threads and observe the behavior.

---

# Chapter 45: Futures, Tasks, and Executors

This chapter dives deeper into how futures are polled and how the executor manages tasks.

## The Poll Function

The `poll` function is the heart of async Rust:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // Two possible return values:
    // Poll::Ready(value) — computation is complete
    // Poll::Pending — computation is not complete, will be polled again later
    
    // When returning Pending, you MUST arrange for cx.waker().wake()
    // to be called when the future can make progress
}
```

**Real-world analogy:** Polling is like checking if your laundry is done. You open the dryer (call poll), and either the clothes are dry (Ready) or still wet (Pending). If they're still wet, you set a timer (register a waker) to check again later.

## Wakers

Wakers通知 executor that a task is ready to make progress:

```rust
use std::task::Waker;

struct MyFuture {
    waker: Option<Waker>,
}

impl Future for MyFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if some_condition() {
            Poll::Ready(())
        } else {
            // Save the waker so we can wake later
            self.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// Later, when the condition changes:
fn wake_task(future: &mut MyFuture) {
    if let Some(waker) = future.waker.take() {
        waker.wake();  // Tell the executor to poll this task again
    }
}
```

## Async Functions as State Machines

When you write an async function, the compiler transforms it into a state machine:

```rust
async fn fetch_and_parse(url: &str) -> Result<String, Error> {
    let response = client.get(url).send().await?;  // State 1
    let html = response.text().await?;              // State 2
    Ok(html)
}
```

The compiler generates something like:

```rust
enum FetchAndParse {
    State0 { url: String },
    State1 { response: Response },
    State2 { body: String },
    Complete,
}

impl Future for FetchAndParse {
    type Output = Result<String, Error>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            State0 { url } => {
                // Start the HTTP request
                // Store the future for the response
                self.state = State1 { response: pending_response };
                Poll::Pending
            }
            State1 { response } => {
                // Check if response is ready
                match response.poll(cx) {
                    Poll::Ready(Ok(body)) => {
                        self.state = State2 { body };
                        Poll::Pending
                    }
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Pending => Poll::Pending,
                }
            }
            // ... more states
        }
    }
}
```

## 📝 Practice Exercises

1. **State Machine:** Write an async function with 3 `.await` points and explain the generated state machine.

2. **Waker Usage:** Create a future that stores a waker and wakes itself after a delay.

3. **Task Lifecycle:** Trace the lifecycle of a spawned task from creation to completion.

---

# Chapter 46: Tokio Internals

Understanding Tokio's internals helps you write more efficient async code.

## The Reactor

Tokio's reactor is an I/O event loop based on epoll (Linux), kqueue (macOS), or IOCP (Windows):

```rust
// Tokio automatically creates a reactor
// You don't need to interact with it directly

// But you can use it for custom I/O:
use tokio::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:8080").await?;
// The reactor monitors this socket for readability/writability
```

## The Timer Wheel

Tokio uses a hierarchical timer wheel for efficient timer management:

```rust
use tokio::time::{sleep, Duration};

// This uses the timer wheel internally
sleep(Duration::from_secs(5)).await;
```

## The Task Queue

Each worker thread has a local task queue. When a task yields (returns `Poll::Pending`), it's placed back in the queue. When a task completes, its dependent tasks are woken.

## Performance Characteristics

- **Task spawn:** ~100ns — very cheap
- **Context switch:** ~1-10μs — cheaper than OS threads
- **Timer resolution:** ~1ms — sufficient for most use cases
- **I/O notification:** ~1-10μs — based on epoll/kqueue

## 📝 Practice Exercises

1. **Benchmark Spawning:** Benchmark spawning 10,000 tasks and measure the time and memory usage.

2. **Timer Accuracy:** Test Tokio's timer accuracy by sleeping for various durations and measuring actual time.

3. **Worker Threads:** Create a runtime with different numbers of worker threads and benchmark performance.

---

# Chapter 47: Async I/O Patterns

This chapter covers common patterns for async I/O operations.

## Concurrent Requests

```rust
use futures::future::join_all;

async fn fetch_all(urls: Vec<String>) -> Vec<Result<String, reqwest::Error>> {
    let client = reqwest::Client::new();
    
    let futures: Vec<_> = urls.iter()
        .map(|url| {
            let client = client.clone();
            let url = url.clone();
            async move {
                client.get(&url).send().await?.text().await
            }
        })
        .collect();
    
    join_all(futures).await
}
```

## Rate-Limited Concurrency

```rust
use tokio::sync::Semaphore;

async fn fetch_with_limit(
    urls: Vec<String>,
    max_concurrent: usize,
) -> Vec<Result<String, reqwest::Error>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let client = reqwest::Client::new();
    
    let futures: Vec<_> = urls.iter()
        .map(|url| {
            let semaphore = semaphore.clone();
            let client = client.clone();
            let url = url.clone();
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                client.get(&url).send().await?.text().await
            }
        })
        .collect();
    
    join_all(futures).await
}
```

## Timeout Handling

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(url: &str) -> Result<String, AppError> {
    let result = timeout(
        Duration::from_secs(30),
        async {
            reqwest::get(url).await?.text().await
        }
    ).await;
    
    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(AppError::Network(e.to_string())),
        Err(_) => Err(AppError::Network("timeout".into())),
    }
}
```

## Stream Processing

```rust
use tokio_stream::StreamExt;

async fn process_stream(urls: Vec<String>) {
    let mut stream = tokio_stream::iter(urls)
        .map(|url| async move {
            reqwest::get(&url).await?.text().await
        })
        .buffer_unordered(10);  // Process 10 at a time
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(text) => process_text(&text),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

## 📝 Practice Exercises

1. **Concurrent Fetching:** Fetch 100 URLs concurrently with a limit of 10 concurrent requests.

2. **Retry with Backoff:** Implement retry logic with exponential backoff using async.

3. **Stream Processing:** Process a stream of database rows concurrently.

---

# Chapter 48: Structured Concurrency with JoinSet

Tokio's `JoinSet` provides structured concurrency — ensuring all spawned tasks complete before proceeding.

## JoinSet Basics

```rust
use tokio::task::JoinSet;

async fn process_items(items: Vec<String>) -> Vec<Result<String, Error>> {
    let mut set = JoinSet::new();
    
    for item in items {
        set.spawn(async move {
            process_item(&item).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    
    results
}
```

### Why JoinSet?

`JoinSet` is better than `join_all` for several reasons:
- Tasks can be added dynamically
- Tasks are automatically cancelled when the set is dropped
- Memory is reclaimed as tasks complete

## JoinSet in FicHub

FicHub could use `JoinSet` for concurrent chapter fetching:

```rust
async fn fetch_chapters_concurrent(
    client: &reqwest::Client,
    chapter_urls: Vec<String>,
) -> Result<Vec<Chapter>, AppError> {
    let mut set = JoinSet::new();
    
    for (i, url) in chapter_urls.into_iter().enumerate() {
        let client = client.clone();
        set.spawn(async move {
            let response = client.get(&url).send().await?;
            let html = response.text().await?;
            let content = parse_chapter_content(&html);
            Ok::<_, AppError>(Chapter {
                chapter_id: i as i32,
                title: format!("Chapter {}", i + 1),
                content,
            })
        });
    }
    
    let mut chapters = Vec::new();
    while let Some(result) = set.join_next().await {
        chapters.push(result??);
    }
    
    chapters.sort_by_key(|c| c.chapter_id);
    Ok(chapters)
}
```

## 📝 Practice Exercises

1. **JoinSet vs join_all:** Benchmark JoinSet vs join_all for 1000 short-lived tasks.

2. **Dynamic Spawning:** Create a JoinSet that dynamically adds tasks based on results from previous tasks.

3. **Cancellation:** Demonstrate how dropping a JoinSet cancels all running tasks.

---

# Chapter 49: Backpressure and Flow Control

Backpressure prevents fast producers from overwhelming slow consumers.

## Channel-Based Backpressure

```rust
use tokio::sync::mpsc;

async fn producer_consumer() {
    let (tx, mut rx) = mpsc::channel::<String>(100);  // Buffer of 100
    
    // Producer
    tokio::spawn(async move {
        for i in 0..1000 {
            let msg = format!("message {}", i);
            tx.send(msg).await.unwrap();  // Blocks if buffer is full
        }
    });
    
    // Consumer
    while let Some(msg) = rx.recv().await {
        process_message(&msg).await;
    }
}
```

## Semaphore-Based Backpressure

```rust
use tokio::sync::Semaphore;

async fn process_with_backpressure(
    items: Vec<String>,
    max_concurrent: usize,
) {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    let futures: Vec<_> = items.iter()
        .map(|item| {
            let sem = semaphore.clone();
            let item = item.clone();
            async move {
                let _permit = sem.acquire().await.unwrap();
                process_item(&item).await
            }
        })
        .collect();
    
    futures::future::join_all(futures).await;
}
```

## Rate Limiting as Backpressure

```rust
use tokio::time::{interval, Duration};

async fn rate_limited_requests(urls: Vec<String>) {
    let mut interval = interval(Duration::from_millis(100));  // 10 req/sec
    
    for url in urls {
        interval.tick().await;  // Wait for next tick
        let response = reqwest::get(&url).await;
        // Process response...
    }
}
```

## 📝 Practice Exercises

1. **Channel Backpressure:** Create a producer-consumer pair with a buffer of 10 and observe what happens when the producer is faster.

2. **Semaphore Limits:** Implement a download manager that limits concurrent downloads to 5.

3. **Rate Limiter:** Build a rate limiter using `tokio::time::Interval` that allows 100 requests per second.

---

# Chapter 50: Async Drop and Cleanup

Rust's `Drop` trait runs synchronously, which can be problematic for async cleanup.

## The Problem

```rust
struct AsyncConnection {
    connection: PgPool,
}

impl Drop for AsyncConnection {
    fn drop(&mut self) {
        // Can't do async work here!
        // self.connection.close().await;  // Compile error
    }
}
```

## Solutions

### Manual Cleanup

```rust
struct AsyncConnection {
    connection: PgPool,
}

impl AsyncConnection {
    async fn close(self) {
        // Async cleanup here
        self.connection.close().await;
    }
}

// Usage:
{
    let conn = AsyncConnection::new();
    // ... use connection ...
    conn.close().await;  // Explicit async cleanup
}  // Drop runs after close()
```

### RAII with async_close

```rust
struct AsyncResource {
    data: Arc<Mutex<Option<Data>>>,
}

impl AsyncResource {
    async fn close(&self) {
        let mut data = self.data.lock().await;
        if let Some(resource) = data.take() {
            resource.async_cleanup().await;
        }
    }
}

impl Drop for AsyncResource {
    fn drop(&mut self) {
        // Synchronous cleanup only
        tracing::debug!("AsyncResource dropped");
    }
}
```

## Graceful Shutdown

```rust
async fn run_server() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(shutdown_rx))
            .await
    });
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.ok();
    shutdown_tx.send(true).ok();
    
    // Wait for server to finish
    server.await.ok();
    
    // Cleanup
    tracing::info!("Shutting down...");
    db_pool.close().await;
}
```

## 📝 Practice Exercises

1. **Manual Cleanup:** Create a struct that requires async cleanup and implement both `Drop` and `close()`.

2. **Graceful Shutdown:** Implement graceful shutdown for a server that waits for in-flight requests to complete.

3. **Resource Leaks:** Write a test that verifies all resources are properly cleaned up when the application shuts down.

