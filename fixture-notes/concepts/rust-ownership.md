# Rust Ownership

Rust's ownership system is the foundation of memory safety without a garbage collector.

## Key Rules

1. Each value in Rust has a single **owner**.
2. When the owner goes out of scope, the value is **dropped**.
3. Ownership can be **moved** or **borrowed**.

## Move Semantics

```rust
let s1 = String::from("hello");
let s2 = s1;
// s1 is no longer valid — ownership moved to s2
```

## Borrowing

Instead of moving ownership, you can **borrow** a reference:

```rust
let s1 = String::from("hello");
let len = calculate_length(&s1);
// s1 is still valid — we only borrowed it
```

### Mutable References

You can have **one** mutable reference OR **any number** of immutable references — but never both at the same time.

## Lifetimes

Lifetimes ensure that references are always valid. The compiler tracks how long references live and prevents dangling references.
