use backtrace::Backtrace;
use serde_derive::Serialize;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::mem;
use std::{env, fs::File, io::Write, path::Path, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone, Copy)]
pub enum Method {
    Panic,
}

#[derive(Debug, Serialize)]
pub struct Report {
    name: String,
    operating_system: String,
    crate_version: String,
    explanation: String,
    cause: String,
    method: Method,
    backtrace: String,
}

impl Report {
    pub fn new(name: &str, version: &str, method: Method, explanation: String, cause: String) -> Self {
        let operating_system = os_info::get().to_string();

        const SKIP_FRAMES_NUM: usize = 8;
        const HEX_WIDTH: usize = mem::size_of::<usize>() + 2;
        const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;

        let mut backtrace = String::new();
        for (idx, frame) in Backtrace::new().frames().iter().skip(SKIP_FRAMES_NUM).enumerate() {
            let ip = frame.ip();
            let _ = write!(backtrace, "\n{idx:4}: {ip:HEX_WIDTH$?}");

            let symbols = frame.symbols();
            if symbols.is_empty() {
                let _ = write!(backtrace, " - <unresolved>");
                continue;
            }

            for (idx, symbol) in symbols.iter().enumerate() {
                if idx != 0 {
                    let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
                }

                if let Some(name) = symbol.name() {
                    let _ = write!(backtrace, " - {name}");
                } else {
                    let _ = write!(backtrace, " - <unknown>");
                }

                if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                    let _ = write!(backtrace, "\n{:3$}at {}:{}", "", file.display(), line, NEXT_SYMBOL_PADDING);
                }
            }
        }

        Self {
            crate_version: version.into(),
            name: name.into(),
            operating_system,
            method,
            explanation,
            cause,
            backtrace,
        }
    }

    pub fn serialize(&self) -> Option<String> {
        toml::to_string_pretty(&self).ok()
    }

    pub fn persist(&self) -> Result<PathBuf, Box<dyn Error + 'static>> {
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let tmp_dir = env::temp_dir();
        let file_name = format!("report-{}.toml", &uuid);
        let file_path = Path::new(&tmp_dir).join(file_name);
        let mut file = File::create(&file_path)?;
        let toml = self.serialize().unwrap();
        file.write_all(toml.as_bytes())?;
        Ok(file_path)
    }
}
