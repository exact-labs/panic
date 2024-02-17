use panic::setup_panic;

fn main() {
    setup_panic! {
        name: "Panic Wrapper",
        short_name: "panic",
        version: env!("CARGO_PKG_VERSION"),
        repository: "https://github.com/exact-labs/panic"
    };

    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
