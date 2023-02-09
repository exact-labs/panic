#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]

pub mod report;
use report::{Method, Report};

use std::io::Result as IoResult;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};

struct Write;
pub struct Metadata {
    pub name: &'static str,
    pub short_name: &'static str,
    pub version: &'static str,
    pub repository: &'static str,
}

#[macro_export]
macro_rules! metadata {
    () => {
        Metadata {
            name: env!("CARGO_PKG_NAME").into(),
            short_name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            repository: env!("CARGO_PKG_HOMEPAGE").into(),
        }
    };
}

#[macro_export]
macro_rules! setup_panic {
    ($meta:expr) => {
        #[allow(unused_imports)]
        use std::panic::{self, PanicInfo};
        #[allow(unused_imports)]
        use $crate::{handle_dump, print_msg, Metadata};

        match $crate::PanicStyle::default() {
            $crate::PanicStyle::Debug => {}
            $crate::PanicStyle::Human => {
                let meta = $meta;

                panic::set_hook(Box::new(move |info: &PanicInfo| {
                    let file_path = handle_dump(&meta, info);
                    print_msg(file_path, &meta).expect("panic: printing error message to console failed");
                }));
            }
        }
    };

    () => {
        $crate::setup_panic!($crate::metadata!());
    };
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PanicStyle {
    Debug,
    Human,
}

impl Default for PanicStyle {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            PanicStyle::Debug
        } else {
            match ::std::env::var("RUST_BACKTRACE") {
                Ok(_) => PanicStyle::Debug,
                Err(_) => PanicStyle::Human,
            }
        }
    }
}

#[cfg(feature = "color")]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

    let stderr_support = concolor::get(concolor::Stream::Stdout);
    let choice = if stderr_support.color() { ColorChoice::Always } else { ColorChoice::Never };
    let stderr = BufferWriter::stderr(choice);
    let mut buffer = stderr.buffer();

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    Write::head(&mut buffer, meta)?;

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    Write::body(&mut buffer, &file_path, meta)?;

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    Write::footer(&mut buffer)?;

    buffer.reset()?;
    stderr.print(&buffer).unwrap();
    Ok(())
}

#[cfg(not(feature = "color"))]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    let mut buffer = std::io::stderr();

    Write::head(&mut buffer, meta)?;
    Write::body(&mut buffer, &file_path, meta)?;
    Write::footer(&mut buffer)?;

    Ok(())
}

impl Write {
    fn head(buffer: &mut impl std::io::Write, meta: &Metadata) -> IoResult<()> {
        let (name, version) = (&meta.name, &meta.version);
        writeln!(buffer, "Well, this is embarrassing.")?;
        writeln!(
            buffer,
            "{name} v{version} had a problem and crashed. To help us diagnose the \
        problem you can send us a crash report.\n"
        )?;

        Ok(())
    }

    fn body<P: AsRef<Path>>(buffer: &mut impl std::io::Write, file_path: &Option<P>, meta: &Metadata) -> IoResult<()> {
        let (short_name, version, repository) = (&meta.short_name, &meta.version, &meta.repository);
        writeln!(
            buffer,
            "We have generated a report file at \"{}\". Submit an \
          issue or email with the subject of \"{short_name} v{version} crash report\" and include the \
          report as an attachment at {repository}/issues.",
            match file_path {
                Some(fp) => format!("{}", fp.as_ref().display()),
                None => "<Failed to store file to disk>".to_string(),
            },
        )?;

        Ok(())
    }

    fn footer(buffer: &mut impl std::io::Write) -> IoResult<()> {
        writeln!(
            buffer,
            "\nWe take privacy seriously, and do not perform any \
          automated error collection. In order to improve the software, we rely on \
          people to submit reports.\nThank you!"
        )?;

        Ok(())
    }
}

pub fn handle_dump(meta: &Metadata, panic_info: &PanicInfo) -> Option<PathBuf> {
    let mut expl = String::new();

    #[cfg(feature = "nightly")]
    let message = panic_info.message().map(|m| format!("{}", m));

    #[cfg(not(feature = "nightly"))]
    let message = match (panic_info.payload().downcast_ref::<&str>(), panic_info.payload().downcast_ref::<String>()) {
        (Some(s), _) => Some(s.to_string()),
        (_, Some(s)) => Some(s.to_string()),
        (None, None) => None,
    };

    let cause = match message {
        Some(m) => m,
        None => "Unknown".into(),
    };

    match panic_info.location() {
        Some(location) => expl.push_str(&format!("Panic occurred in file '{}' at line {}\n", location.file(), location.line())),
        None => expl.push_str("Panic location unknown.\n"),
    }

    let report = Report::new(&meta.short_name, &meta.version, Method::Panic, expl, cause);

    match report.persist() {
        Ok(f) => Some(f),
        Err(_) => {
            eprintln!("{}", report.serialize().unwrap());
            None
        }
    }
}
