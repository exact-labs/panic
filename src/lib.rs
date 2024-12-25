#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]

#[cfg(feature = "color")]
pub use anstyle::AnsiColor as Color;
pub mod report;

use report::{Method, Report};
use text_placeholder::Template;

use std::{
    borrow::Cow,
    collections::HashMap,
    io::Result as IoResult,
    panic::PanicHookInfo,
    path::{Path, PathBuf},
};

struct Writer<'w> {
    meta: &'w Metadata,
    table: HashMap<&'w str, &'w str>,
    pub(crate) buffer: &'w mut dyn std::io::Write,
}

pub struct Metadata {
    pub messages: Messages,
    pub name: Cow<'static, str>,
    pub short_name: Cow<'static, str>,
    pub version: Cow<'static, str>,
    pub repository: Cow<'static, str>,
}

#[derive(Clone)]
pub struct Messages {
    pub head: (Option<Cow<'static, str>>, Color),
    pub body: (Option<Cow<'static, str>>, Color),
    pub footer: (Option<Cow<'static, str>>, Color),
}

#[macro_export]
macro_rules! setup_panic {
    (@field_arm colors $value:expr, $meta:expr) => {
        $meta.messages.head.1 = $value.0;
        $meta.messages.body.1 = $value.1;
        $meta.messages.footer.1 = $value.2;
    };
    (@field_arm $field:ident $value:expr, $meta:expr) => {
        let value: Cow<'static, str> = $value.into();
        $meta.messages.$field.0 = Some(value).filter(|val| !val.is_empty());
    };
    (
        $(name: $name:expr,)?
        $(short_name: $short_name:expr,)?
        $(version: $version:expr,)?
        $(repository: $repository:expr,)?
        $(messages: {
            $(colors: $colors:expr,)?
            $(head: $head:expr,)?
            $(body: $body:expr,)?
            $(footer: $footer:expr)?
        })?
    ) => {
        #[allow(unused_imports)]
        use std::{borrow::Cow, panic::{self, PanicHookInfo}};
        #[allow(unused_imports)]
        use $crate::{handle_dump, print_msg, Color, Messages, Metadata};

        let mut msg = Messages {
            head: (None, Color::Red),
            body: (None, Color::White),
            footer: (None, Color::Green),
        };

        let mut meta = Metadata {
            messages: msg,
            name: env!("CARGO_PKG_NAME").into(),
            short_name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            repository: env!("CARGO_PKG_REPOSITORY").into(),
        };

        $(meta.name=$name.into();)?
        $(meta.short_name=$short_name.into();)?
        $(meta.version=$version.into();)?
        $(meta.repository=$repository.into();)?

        $(
            $($crate::setup_panic!(@field_arm head $head, meta);)?
            $($crate::setup_panic!(@field_arm body $body, meta);)?
            $($crate::setup_panic!(@field_arm footer $footer, meta);)?
            $($crate::setup_panic!(@field_arm colors $colors, meta);)?
        )?

        match $crate::PanicStyle::default() {
            $crate::PanicStyle::Debug => {}
            $crate::PanicStyle::Human => {
                panic::set_hook(Box::new(move |info: &PanicHookInfo| {
                    let file_path = handle_dump(&meta, info);
                    print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
                }));
            }
        }
    };
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
    let stderr = anstream::stderr();
    let mut stderr = stderr.lock();
    let mut writer = Writer::new(&mut stderr, file_path, meta);

    write!(writer.buffer, "{}", meta.messages.head.1.render_fg())?;
    writer.head()?;
    write!(writer.buffer, "{}", anstyle::Reset.render())?;

    write!(writer.buffer, "{}", meta.messages.body.1.render_fg())?;
    writer.body()?;
    write!(writer.buffer, "{}", anstyle::Reset.render())?;

    write!(writer.buffer, "{}", meta.messages.footer.1.render_fg())?;
    writer.footer()?;
    write!(writer.buffer, "{}", anstyle::Reset.render())?;

    Ok(())
}

#[cfg(not(feature = "color"))]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    let stderr = std::io::stderr();
    let mut stderr = stderr.lock();
    let mut writer = Writer::new(&mut stderr, file_path, meta);

    writer.head()?;
    writer.body()?;
    writer.footer()?;

    Ok(())
}

impl<'w> Writer<'w> {
    fn new<P: AsRef<Path>>(buffer: &'w mut impl std::io::Write, file_path: Option<P>, meta: &'w Metadata) -> Self {
        let mut table = HashMap::new();

        let file_path = match file_path {
            Some(fp) => format!("{}", fp.as_ref().display()),
            None => "<Failed to store file to disk>".to_string(),
        };

        table.insert("name", meta.name.as_ref());
        table.insert("version", meta.version.as_ref());
        table.insert("short_name", meta.short_name.as_ref());
        table.insert("repository", meta.repository.as_ref());
        table.insert("file_path", Box::leak(Box::new(file_path)));

        Self { buffer, meta, table }
    }

    fn head(&mut self) -> IoResult<()> {
        let (name, version) = (&self.meta.name, &self.meta.version);

        if let Some(head) = &self.meta.messages.head.0 {
            let tmpl = Template::new_with_placeholder(&head, "%(", ")");
            writeln!(self.buffer, "{}", tmpl.fill_with_hashmap(&self.table))?;
        } else {
            writeln!(self.buffer, "Well, this is embarrassing.")?;
            writeln!(
                self.buffer,
                "{name} v{version} had a problem and crashed. To help us diagnose the \
            problem you can send us a crash report.\n"
            )?;
        }

        Ok(())
    }

    fn body(&mut self) -> IoResult<()> {
        let (short_name, version, repository) = (&self.meta.short_name, &self.meta.version, &self.meta.repository);

        if let Some(body) = &self.meta.messages.body.0 {
            let tmpl = Template::new_with_placeholder(&body, "%(", ")");
            writeln!(self.buffer, "{}", tmpl.fill_with_hashmap(&self.table))?;
        } else {
            writeln!(
                self.buffer,
                "We have generated a report file at \"{}\". Submit an \
          issue or email with the subject of \"{short_name} v{version} crash report\" and include the \
          report as an attachment at {repository}/issues.",
                self.table.get("file_path").unwrap(),
            )?;
        }

        Ok(())
    }

    fn footer(&mut self) -> IoResult<()> {
        if let Some(footer) = &self.meta.messages.footer.0 {
            let tmpl = Template::new_with_placeholder(&footer, "%(", ")");
            writeln!(self.buffer, "{}", tmpl.fill_with_hashmap(&self.table))?;
        } else {
            writeln!(
                self.buffer,
                "\nWe take privacy seriously, and do not perform any \
          automated error collection. In order to improve the software, we rely on \
          people to submit reports.\nThank you!"
            )?;
        }
        Ok(())
    }
}

pub fn handle_dump(meta: &Metadata, panic_info: &PanicHookInfo) -> Option<PathBuf> {
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
