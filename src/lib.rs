#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]

pub mod report;
use report::{Method, Report};

use std::borrow::Cow;
use std::io::Result as IoResult;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};

struct Write;
pub struct Metadata {
    pub name: Cow<'static, str>,
    pub short_name: Cow<'static, str>,
    pub version: Cow<'static, str>,
    pub repository: Cow<'static, str>,
}

#[macro_export]
macro_rules! setup_from_metadata {
    () => {
        $crate::setup_panic!(
            name: env!("CARGO_PKG_NAME"),
            short_name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            repository: env!("CARGO_PKG_REPOSITORY")
        )
    };
}

#[macro_export]
macro_rules! setup_panic {
    ($($field:ident : $value:expr),*) => {{
        #[allow(unused_imports)]
        use std::panic::{self, PanicInfo};
        #[allow(unused_imports)]
        use $crate::{handle_dump, print_msg, Metadata};

        match $crate::PanicStyle::default() {
            $crate::PanicStyle::Debug => {}
            $crate::PanicStyle::Human => {
                let meta = Metadata {
                    $($field: $value.into()),*
                };

                panic::set_hook(Box::new(move |info: &PanicInfo| {
                    let file_path = handle_dump(&meta, info);
                    print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
                }));
            }
        }
    }};
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PanicStyle {
    Debug,
    Human,
}

#[cfg(feature = "only-release")]
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

#[cfg(not(feature = "only-release"))]
impl Default for PanicStyle {
    fn default() -> Self {
        match ::std::env::var("RUST_BACKTRACE") {
            Ok(_) => PanicStyle::Debug,
            Err(_) => PanicStyle::Human,
        }
    }
}

#[cfg(feature = "color")]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    use std::io::Write as _;

    let stderr = anstream::stderr();
    let mut stderr = stderr.lock();

    write!(stderr, "{}", anstyle::AnsiColor::Red.render_fg())?;
    Write::head(&mut stderr, meta)?;
    write!(stderr, "{}", anstyle::Reset.render())?;

    write!(stderr, "{}", anstyle::AnsiColor::White.render_fg())?;
    Write::body(&mut stderr, &file_path, meta)?;
    write!(stderr, "{}", anstyle::Reset.render())?;

    write!(stderr, "{}", anstyle::AnsiColor::Green.render_fg())?;
    Write::footer(&mut stderr)?;
    write!(stderr, "{}", anstyle::Reset.render())?;

    Ok(())
}

#[cfg(not(feature = "color"))]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    let stderr = std::io::stderr();
    let mut stderr = stderr.lock();

    Write::head(&mut stderr, meta)?;
    Write::body(&mut stderr, &file_path, meta)?;
    Write::footer(&mut stderr)?;

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
