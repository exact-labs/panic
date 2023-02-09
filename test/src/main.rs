fn main() {
    exact_panic::setup_panic!(Metadata {
        name: "The justjs runtime",
        short_name: "justjs",
        version: env!("CARGO_PKG_VERSION"),
        repository: "https://github.com/exact-rs/just"
    });

    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
