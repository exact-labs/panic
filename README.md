# panic

Panic messages for humans. Handles panics by calling
[`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
to make errors nice for humans. This is a fork of the original [human-panic](https://github.com/rust-cli/human-panic) crate.

## Usage

```rust
use panic::setup_from_metadata;

fn main() {
	 setup_from_metadata!();

	 println!("A normal log message");
	 panic!("OMG EVERYTHING IS ON FIRE!!!");
}
```

It only displays a human-friendly panic message in release mode:

```sh
$ cargo run --release
```

## Installation

```sh
$ cargo add panic
```
