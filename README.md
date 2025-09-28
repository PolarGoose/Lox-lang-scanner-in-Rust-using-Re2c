My personal implementation of the scanner/lexer for the Lox language from the book [Bob Nystrom - Crafting Interpreters](https://craftinginterpreters.com/).<br>
This repository shows how to:
* Create an iterator that parses lexemes one at a time using Re2c and Rust.
* Track the positions of lexemes within the source text to be able to write error or diagnostic messages to the user.
* Integrate Re2c into a Rust project.

# Why use Re2c for this task?
* Re2c generates a static parsing state machine without any heap allocations. It can be useful for performance-critical applications and systems where heap allocations are not desirable, like embedded systems.
* I wanted to try the Re2c tool for some non-trivial use case to evaluate its capabilities.

# How to use the code
The whole implementation of the scanner is located in the [src/lox_language_scanner.re2c.rs](https://github.com/PolarGoose/Lox-lang-scanner-in-Rust-using-Re2c/blob/main/src/lox_language_scanner.re2c.rs) file.

## Scanner usage example
```
let lox_src = r#"
    // variables and math
    var x = 42;
    var y = 3.14;
    print "hello, world";
    if (x >= y) {
        x = x + 1;
    } else {
        y = y - 1;
    }"#;

// Scanner implements Iterator. Each iteration returns Result<Token>
for token in Scanner::new(lox_src) {
    println!("{:?}", token);
    
    // If token is Err, then it means that the parsing error happened.
    if token.is_err() {
        // Handle UnexpectedSymbolError
        
        // In case of an error, we can continue parsing.
        // In this example we just break the loop.
        break;
    }
}
```

# How to build this repository
The build only works on Windows because of how the `build.rs` script is implemented. The `build.rs` script can be easily adapted to work on Linux.

# References
[Crafting Interpreters - ch. 4 - Scanning](https://craftinginterpreters.com/scanning.html) - how to create a Lox language lexer using Java.