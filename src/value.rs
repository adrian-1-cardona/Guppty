// === value.rs ===
// Values are the actual "things" that exist when your program is running.
//
// Think of it like this:
//   - In your code you WRITE: "Hello World!"
//   - When the program RUNS, that becomes an actual Value — a piece of text
//     sitting in the computer's memory.
//
// Right now Guppty only has one type of value: text (we call it a String).
// Later you could add numbers, true/false, lists, etc.!

/// These are all the types of values that can exist while a Guppty program runs.
/// It's like asking "what kinds of things can Guppty hold in its hands?"
#[derive(Debug, Clone)]
pub enum Value {
    // A piece of text, like "Hello World!"
    GuppyString(String),

    // "Nothing" — when something doesn't give back a value.
    // Like when out("hi") prints "hi" — it doesn't really RETURN anything useful.
    Nothing,
}
