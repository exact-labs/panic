use panic::{create_messages, setup_panic};

fn main() {
    setup_panic! {
        name: "Panic Wrapper",
        short_name: "panic",
        version: env!("CARGO_PKG_VERSION"),
        repository: "https://github.com/exact-labs/panic",
        message: create_messages! {
            head: "Well, this is embarrassing.\n%(name) v%(version) had a problem and crashed. To help us diagnose the problem you can send us a crash report\n",
            body: "We have generated a report file at \"%(file_path)\". Submit an issue or email with the subject of \"%(name) v%(version) crash report\" and include the report as an attachment at %(repository).\n",
            footer: "We take privacy seriously, and do not perform any automated error collection. In order to improve the software, we rely on people to submit reports. \nThank you!"
        }
    };

    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
