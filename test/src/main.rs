use panic::setup_panic;

fn main() {
    setup_panic! {
        name: "Panic Wrapper",
        short_name: "panic",
        version: env!("CARGO_PKG_VERSION"),
        repository: "https://github.com/exact-labs/panic",
        messages: {
            colors: (Color::Red, Color::White, Color::Green),
            head: "Well, this is embarrassing. %(name) v%(version) had a problem and crashed. \nTo help us diagnose the problem you can send us a crash report\n",
            body: "We have generated a report file at \"%(file_path)\". \nSubmit an issue or email with the subject of \"%(name) v%(version) crash report\" and include the report as an attachment at %(repository).\n",
            footer: "We take privacy seriously, and do not perform any automated error collection. \nIn order to improve the software, we rely on people to submit reports. Thank you!"
        }
    };

    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
