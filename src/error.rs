#![allow(dead_code)]
use chrono::DateTime;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorContext {
    pub error: Error,
    pub timestamp: DateTime<Local>,
    pub suggested_action: String,
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.timestamp.format("%Y-%m-%d %H:%M:%S"), self.error)
    }
}

impl From<Error> for ErrorContext {
    fn from(value: Error) -> Self {
        ErrorContext { error: value, timestamp: Local::now(), suggested_action: "N/A".to_string() }
    }
}

impl From<ErrorData> for ErrorContext {
    fn from(value: ErrorData) -> Self {
        ErrorContext { error: value.into(), timestamp: Local::now(), suggested_action: "N/A".to_string() }
    }
}

impl ErrorContext {
    pub fn new(error: Error, timestamp: DateTime<Local>, suggested_action: String) -> ErrorContext {
        ErrorContext { error, timestamp, suggested_action }
    }

    pub fn builder() -> ErrorContextBuilder {
        ErrorContextBuilder::new()
    }

    pub fn timestamp_string(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Debug)]
pub struct ErrorContextBuilder {
    error: Option<Error>,
    timestamp: Option<DateTime<Local>>,
    suggested_action: Option<String>,
}

impl ErrorContextBuilder {
    pub fn new() -> ErrorContextBuilder {
        ErrorContextBuilder { error: None, timestamp: None, suggested_action: None }
    }

    pub fn error(mut self, error: Error) -> ErrorContextBuilder {
        self.error = Some(error);
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Local>) -> ErrorContextBuilder {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn suggested_action(mut self, suggested_action: &str) -> ErrorContextBuilder {
        self.suggested_action = Some(suggested_action.to_owned());
        self
    }

    pub fn build(self) -> ErrorContext {
        ErrorContext {
            error: self.error.unwrap_or(Error::other("N/A", "N/A")),
            timestamp: self.timestamp.unwrap_or(Local::now()),
            suggested_action: self.suggested_action.unwrap_or("N/A".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Error {
    Service(ErrorData),
    FileSystem(ErrorData),
    System(ErrorData),
    External(ErrorData),
    Other(ErrorData),
}

impl From<ErrorData> for Error {
    fn from(info: ErrorData) -> Self {
        Error::new(info.msg, info.source, info.operation)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::file_system(err.to_string(), err.kind().to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_error = |e: &ErrorData| format!("{}: During \"{}\" - {}", e.source, e.operation, e.msg);
        match self {
            Error::Service(info) => write!(f, "{}", format_error(info)),
            Error::FileSystem(info) => write!(f, "{}", format_error(info)),
            Error::System(info) => write!(f, "{}", format_error(info)),
            Error::External(info) => write!(f, "{}", format_error(info)),
            Error::Other(info) => write!(f, "{}", format_error(info)),
        }
    }
}

impl Error {
    pub fn new<S: Into<String>>(msg: S, source: S, operation: S) -> Error {
        Error::Other(ErrorData { msg: msg.into(), source: source.into(), operation: operation.into() })
    }

    pub fn other<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::Other(ErrorData { msg: msg.into(), source: "Unknown".into(), operation: operation.into() })
    }

    pub fn mods_service<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::Service(ErrorData {
            msg: msg.into(),
            source: "Mods Service".into(),
            operation: operation.into(),
        })
    }

    pub fn profile_service<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::Service(ErrorData {
            msg: msg.into(),
            source: "Profile Service".into(),
            operation: operation.into(),
        })
    }

    pub fn session_service<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::Service(ErrorData {
            msg: msg.into(),
            source: "Session Service".into(),
            operation: operation.into(),
        })
    }

    pub fn ui_service<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::Service(ErrorData {
            msg: msg.into(),
            source: "UI Service".into(),
            operation: operation.into(),
        })
    }

    pub fn file_system<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::FileSystem(ErrorData {
            msg: msg.into(),
            source: "File System".into(),
            operation: operation.into(),
        })
    }

    pub fn system<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::System(ErrorData { msg: msg.into(), source: "System".into(), operation: operation.into() })
    }

    pub fn external<S: Into<String>>(msg: S, operation: S) -> Error {
        Error::External(ErrorData { msg: msg.into(), source: "External".into(), operation: operation.into() })
    }

    pub fn error_message(self) -> String {
        match self {
            Error::Service(info) => info.msg,
            Error::FileSystem(info) => info.msg,
            Error::System(info) => info.msg,
            Error::External(info) => info.msg,
            Error::Other(info) => info.msg,
        }
    }

    pub fn error_source(self) -> String {
        match self {
            Error::Service(info) => info.source,
            Error::FileSystem(info) => info.source,
            Error::System(info) => info.source,
            Error::External(info) => info.source,
            Error::Other(info) => info.source,
        }
    }

    pub fn error_operation(self) -> String {
        match self {
            Error::Service(info) => info.operation,
            Error::FileSystem(info) => info.operation,
            Error::System(info) => info.operation,
            Error::External(info) => info.operation,
            Error::Other(info) => info.operation,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorData {
    pub msg: String,
    pub source: String,
    pub operation: String,
}

impl ErrorData {
    pub fn new<S: Into<String>>(msg: S, source: S, operation: S) -> ErrorData {
        ErrorData { msg: msg.into(), source: source.into(), operation: operation.into() }
    }

    pub fn builder() -> ErrorInfoBuilder {
        ErrorInfoBuilder::new()
    }
}

#[derive(Debug)]
pub struct ErrorInfoBuilder {
    msg: Option<String>,
    source: Option<String>,
    operation: Option<String>,
}

impl ErrorInfoBuilder {
    pub fn new() -> ErrorInfoBuilder {
        ErrorInfoBuilder { msg: None, source: None, operation: None }
    }

    pub fn msg<S: Into<String>>(mut self, msg: S) -> ErrorInfoBuilder {
        self.msg = Some(msg.into());
        self
    }

    pub fn source<S: Into<String>>(mut self, source: S) -> ErrorInfoBuilder {
        self.source = Some(source.into());
        self
    }

    pub fn operation<S: Into<String>>(mut self, operation: S) -> ErrorInfoBuilder {
        self.operation = Some(operation.into());
        self
    }

    pub fn build(self) -> ErrorData {
        ErrorData {
            msg: self.msg.unwrap_or("N/A".into()),
            source: self.source.unwrap_or("N/A".into()),
            operation: self.operation.unwrap_or("N/A".into()),
        }
    }
}
