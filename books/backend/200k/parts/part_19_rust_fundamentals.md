# Extended Content: Comprehensive Tutorial Sections

---

# Chapter: Rust Fundamentals — A Complete Tutorial

## Variables and Mutability

In Rust, variables are immutable by default. This is a deliberate design choice that encourages safer code. When you declare a variable with `let`, you cannot reassign it. To make a variable mutable, you must explicitly use the `mut` keyword.

```rust
fn main() {
    let x = 5;
    // x = 6;  // This would cause a compile error
    
    let mut y = 5;
    y = 6;  // This is allowed
    println!("y = {}", y);  // y = 6
}
```

**Real-world analogy:** Think of immutable variables as constants written in stone — they cannot be changed once created. Mutable variables are like variables written on a whiteboard — they can be erased and rewritten. Rust defaults to "stone" because it's safer. You have to explicitly opt into "whiteboard" mode with `mut`.

This design philosophy extends throughout Rust. By making things immutable by default, the compiler can catch many bugs at compile time. When you see `let mut`, you know that this variable is designed to change, and you should pay extra attention to how and when it changes.

### Shadowing

Rust allows you to declare a new variable with the same name as a previous variable. This is called shadowing, and it's different from mutation:

```rust
fn main() {
    let x = 5;
    let x = x + 1;      // x is now 6
    let x = x * 2;      // x is now 12
    println!("x = {}", x);  // x = 12
    
    // Shadowing with type change
    let spaces = "   ";      // &str
    let spaces = spaces.len();  // usize
    println!("spaces = {}", spaces);  // spaces = 3
}
```

Shadowing is useful when you need to transform a value or change its type. Unlike mutation, shadowing creates a completely new variable — the old variable is no longer accessible.

### Constants

Constants are similar to immutable variables but with some differences:

```rust
const MAX_POINTS: u32 = 100_000;
const PI: f64 = 3.141592653589793;

fn main() {
    println!("Max points: {}", MAX_POINTS);
    println!("PI: {}", PI);
}
```

Constants must:
1. Have their type explicitly annotated
2. Use `SCREAMING_SNAKE_CASE`
3. Be assigned a compile-time constant expression
4. Can be declared in any scope, including global scope

## Data Types

Rust is statically typed, which means the compiler must know the type of every variable at compile time. However, Rust can usually infer the type from the context, so you don't always need to specify it explicitly.

### Scalar Types

Scalar types represent a single value. Rust has four primary scalar types:

#### Integers

```rust
let a: i8 = 127;        // 8-bit signed
let b: u8 = 255;         // 8-bit unsigned
let c: i16 = 32767;      // 16-bit signed
let d: u16 = 65535;      // 16-bit unsigned
let e: i32 = 2147483647;  // 32-bit signed
let f: u32 = 4294967295;  // 32-bit unsigned
let g: i64 = 9223372036854775807;  // 64-bit signed
let h: u64 = 18446744073709551615; // 64-bit unsigned

// Default integer type is i32
let x = 42;  // i32

// Underscores for readability
let million = 1_000_000;
let binary = 0b1111_0000;
let hex = 0xFF;
let octal = 0o77;
```

Integer overflow in debug mode causes a panic. In release mode, it wraps around:

```rust
let x: u8 = 255;
// let y: u8 = x + 1;  // Panics in debug mode, wraps to 0 in release

// Safe alternatives
let y = x.wrapping_add(1);    // Wraps: 0
let y = x.checked_add(1);     // Returns None
let y = x.overflowing_add(1); // Returns (0, true)
let y = x.saturating_add(1);  // Saturates: 255
```

#### Floating-Point

```rust
let x = 2.0;      // f64 (default)
let y: f32 = 3.0;  // f32

// Scientific notation
let million = 1e6;
let tiny = 1.5e-10;

// Special values
let inf = f64::INFINITY;
let nan = f64::NAN;
```

#### Boolean

```rust
let t = true;
let f: bool = false;

// Logical operators
let and = true && false;   // false
let or = true || false;    // true
let not = !true;           // false
```

#### Character

```rust
let c = 'z';
let z: char = 'ℤ';
let heart = '❤';
let chinese = '中';

// Character is 4 bytes (Unicode)
println!("Size of char: {}", std::mem::size_of::<char>());  // 4
```

### Compound Types

#### Tuple

A tuple groups multiple values of different types:

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;  // Destructuring
let five_hundred = tup.0;  // Index access
let six_point_four = tup.1;
let one = tup.2;

// Unit type (empty tuple)
let unit = ();
```

#### Array

An array is a fixed-size collection of elements of the same type:

```rust
let a = [1, 2, 3, 4, 5];  // [i32; 5]
let a: [i32; 5] = [1, 2, 3, 4, 5];
let a = [3; 5];  // [3, 3, 3, 3, 3]

let first = a[0];
let second = a[1];

// Arrays are stack-allocated and have fixed size
// For dynamic size, use Vec<T>
```

## Functions

Functions are defined with `fn`:

```rust
fn add(x: i32, y: i32) -> i32 {
    x + y  // No semicolon = return value
}

fn greet(name: &str) {
    println!("Hello, {}!", name);  // Semicolon = no return value
}

fn main() {
    let sum = add(3, 4);
    greet("World");
    println!("3 + 4 = {}", sum);
}
```

### Parameters and Arguments

Parameters are the variables listed in the function signature. Arguments are the actual values passed to the function:

```rust
fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {}{}", value, unit_label);
}

fn main() {
    print_labeled_measurement(5, 'h');  // 5 is the argument for value
}
```

### Statements and Expressions

Statements execute actions but don't return values. Expressions evaluate to a value:

```rust
fn main() {
    // Statement (no return value)
    let y = {
        let x = 3;
        x + 1  // Expression (returns 4)
    };
    
    println!("y = {}", y);  // y = 4
    
    // Function call is an expression
    let result = add(3, 4);
    
    // If expression
    let condition = true;
    let number = if condition { 5 } else { 6 };
    println!("number = {}", number);  // number = 5
}
```

## Control Flow

### If Expressions

```rust
fn main() {
    let number = 7;
    
    if number < 5 {
        println!("condition was true");
    } else if number < 10 {
        println!("condition was also true");
    } else {
        println!("condition was false");
    }
    
    // If is an expression
    let result = if number > 0 { "positive" } else { "non-positive" };
    println!("Number is {}", result);
}
```

### Loops

```rust
fn main() {
    // loop
    let mut count = 0;
    loop {
        count += 1;
        if count == 5 {
            break;
        }
        println!("count = {}", count);
    }
    
    // loop with return value
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2;
        }
    };
    println!("result = {}", result);  // result = 20
    
    // while
    let mut number = 3;
    while number != 0 {
        println!("{}!", number);
        number -= 1;
    }
    println!("LIFEOFF!!!");
    
    // for
    let a = [10, 20, 30, 40, 50];
    for element in a {
        println!("the value is: {}", element);
    }
    
    // for with range
    for number in 1..4 {
        println!("{}!", number);
    }
    
    // for with rev
    for number in (1..4).rev() {
        println!("{}!", number);
    }
}
```

## Ownership in Detail

Ownership is Rust's most distinctive feature. Every value has exactly one owner. When the owner goes out of scope, the value is dropped.

```rust
fn main() {
    {
        let s = String::from("hello");  // s comes into scope
        // Use s
    }  // s goes out of scope and is dropped
    
    // s is no longer valid
}
```

### Move Semantics

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is moved to s2
    
    // println!("{}", s1);  // Error: s1 was moved
    println!("{}", s2);     // Works: s2 owns the value
}
```

**Real-world analogy:** Ownership is like a library book checkout system. Only one person can check out a book at a time. When you "check out" a book (move it to a new variable), the original person no longer has it. When you're done with the book (the variable goes out of scope), it's returned to the library (freed).

### Clone

If you want to duplicate the data, use `clone`:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Deep copy
    
    println!("s1 = {}, s2 = {}", s1, s2);  // Both valid
}
```

### Copy Trait

Types that implement the `Copy` trait are copied instead of moved:

```rust
fn main() {
    let x = 5;
    let y = x;  // Copy, not move
    
    println!("x = {}, y = {}", x, y);  // Both valid
}
```

Common Copy types: `i32`, `f64`, `bool`, `char`, tuples of Copy types.

### Borrowing

Borrowing allows you to reference data without taking ownership:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);  // Borrow s1
    println!("The length of '{}' is {}.", s1, len);  // s1 still valid
}
```

### Mutable Borrowing

```rust
fn change(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s);  // "hello, world"
}
```

### Lifetimes

Lifetimes ensure references don't outlive the data they point to:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(&string1, &string2);
        println!("Longest: {}", result);  // Works
    }
    // Can't use result here
}
```

## Structs

Structs group related data together:

```rust
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

fn main() {
    let user = User {
        username: String::from("someone"),
        email: String::from("someone@example.com"),
        active: true,
        sign_in_count: 1,
    };
    
    println!("Username: {}", user.username);
    
    // Mutable struct
    let mut user = User {
        username: String::from("someone"),
        email: String::from("someone@example.com"),
        active: true,
        sign_in_count: 1,
    };
    user.email = String::from("newemail@example.com");
}
```

### Tuple Structs

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
    
    println!("Black: {} {} {}", black.0, black.1, black.2);
}
```

### Unit-Like Structs

```rust
struct AlwaysEqual;

fn main() {
    let _subject = AlwaysEqual;
}
```

## Enums

Enums define a type by enumerating its possible variants:

```rust
enum IpAddrKind {
    V4,
    V6,
}

struct IpAddr {
    kind: IpAddrKind,
    address: String,
}

fn main() {
    let home = IpAddr {
        kind: IpAddrKind::V4,
        address: String::from("127.0.0.1"),
    };
    
    let loopback = IpAddr {
        kind: IpAddrKind::V6,
        address: String::from("::1"),
    };
}
```

### Enums with Data

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit"),
            Message::Move { x, y } => println!("Move to {}, {}", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: {}, {}, {}", r, g, b),
        }
    }
}

fn main() {
    let msg = Message::Write(String::from("hello"));
    msg.call();
}
```

### Option<T>

```rust
enum Option<T> {
    None,
    Some(T),
}

fn main() {
    let some_number: Option<i32> = Some(5);
    let some_string: Option<String> = Some(String::from("hello"));
    let absent_number: Option<i32> = None;
    
    // Pattern matching
    match some_number {
        Some(n) => println!("Number: {}", n),
        None => println!("No number"),
    }
    
    // unwrap
    let n = some_number.unwrap();
    
    // unwrap_or
    let n = absent_number.unwrap_or(0);
}
```

## Traits

Traits define shared behavior:

```rust
trait Summary {
    fn summarize(&self) -> String;
    
    // Default implementation
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..50])
    }
}

struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}, by {}", self.title, self.author)
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

fn main() {
    let article = Article {
        title: String::from("My Article"),
        author: String::from("John"),
        content: String::from("Lorem ipsum..."),
    };
    
    let tweet = Tweet {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably"),
    };
    
    println!("Article: {}", article.summarize());
    println!("Tweet: {}", tweet.summarize());
}
```

## Generics

Generics allow you to write code that works with multiple types:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    
    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    let result = largest(&numbers);
    println!("Largest number: {}", result);
    
    let chars = vec!['y', 'm', 'a', 'q'];
    let result = largest(&chars);
    println!("Largest char: {}", result);
}
```

## Error Handling

### Result<T, E>

```rust
use std::fs;

fn read_username_from_file() -> Result<String, std::io::Error> {
    let username = fs::read_to_string("username.txt")?;
    Ok(username.trim().to_string())
}

fn main() {
    match read_username_from_file() {
        Ok(username) => println!("Username: {}", username),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Custom Errors

```rust
#[derive(Debug)]
enum AppError {
    NotFound(String),
    PermissionDenied,
    NetworkError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::PermissionDenied => write!(f, "Permission denied"),
            AppError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
```

## Closures

Closures are anonymous functions that can capture variables from their scope:

```rust
fn main() {
    let add_one = |x| x + 1;
    println!("5 + 1 = {}", add_one(5));
    
    let name = String::from("Alice");
    let greet = || println!("Hello, {}!", name);
    greet();
    
    let mut list = vec![1, 2, 3];
    let mut push_value = || list.push(4);
    push_value();
    println!("{:?}", list);  // [1, 2, 3, 4]
}
```

## Iterators

Iterators provide a way to process sequences:

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];
    
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    println!("Doubled: {:?}", doubled);
    
    let evens: Vec<&i32> = v.iter().filter(|x| *x % 2 == 0).collect();
    println!("Evens: {:?}", evens);
    
    let sum: i32 = v.iter().sum();
    println!("Sum: {}", sum);
}
```

