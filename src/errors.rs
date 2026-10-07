//! Error-handling utilities.

use std::{
    cell::Cell,
    error::Error,
    fmt::{self, Display, Formatter},
    range::Range,
    result,
};

/// The [`Result`][result::Result] type alias used throughout most of the compiler. Defaults to [`HandledError`] for it's `Err` variant.
pub type Result<T, E = HandledError> = result::Result<T, E>;

/// Indicates that an error occurred, and detailed information was provided to an [`ErrorHandler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HandledError;

impl Display for HandledError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        "Detailed error was printed to stderr".fmt(f)
    }
}

impl Error for HandledError {}

/// A trait for user-reportable diagnostics, including errors and warnings.
pub trait Diagnostic {
    /// Constructs a report from this diagnostic.
    fn report(self) -> Report;
}

/// A textual report generated from a diagnostic.
/// See [`Diagnostic`].
#[derive(Debug)]
pub struct Report {
    /// The name of the diagnostic.
    pub name: String,
    /// An optional label to be attached below the highlighted source span.
    pub label: Option<String>,
    /// A list of notes to be displayed after the main report.
    pub notes: Vec<String>,
}

impl Report {
    /// Constructs a new [`Report`].
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: None,
            notes: Vec::new(),
        }
    }

    /// Sets the label of the [`Report`].
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Attaches a note to the [`Report`].
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl Diagnostic for Report {
    fn report(self) -> Report {
        self
    }
}

/// The type of the callback functions wrapped by [`ErrorHandler`].
pub type HandlerCallback<'cb> = &'cb dyn Fn(Report, Range<u32>);

/// A utility type to handle configurable, recoverable diagnostic reporting.
///
/// The primary usecase is reporting errors via [`err()`][Self::err()] and returning the [`HandledError`].
/// This type internally tracks if any errors have been reported, which can be used through [`checked()`][`Self::checked()`].
///
/// Cloning this type is cheap, and it could implement `Copy` but doesn't for similar reasons to iterators.
#[derive(Clone)]
pub struct ErrorHandler<'cb> {
    f: HandlerCallback<'cb>,
    has_err: Cell<bool>,
}

impl<'cb> ErrorHandler<'cb> {
    /// Constructs a new `ErrorHandler` with the provided reporting callback.
    pub const fn new(f: HandlerCallback<'cb>) -> Self {
        Self {
            f,
            has_err: Cell::new(false),
        }
    }

    /// Reports the provided diagnostic and produces a [`HandledError`] for the caller to use.
    pub fn report(&self, diagnostic: impl Diagnostic, span: impl Into<Range<u32>>) -> HandledError {
        let report = diagnostic.report();
        self.has_err.set(true);
        (self.f)(report, span.into());
        HandledError
    }

    /// Returns the provided value wrapped in [`Ok`], or a [`HandledError`] if this handler has reported any errors.
    ///
    /// # Errors
    /// [`HandledError`] if this handler has reported any errors.
    pub fn checked<T>(&self, val: T) -> Result<T> {
        if self.has_err.get() {
            Err(HandledError)
        } else {
            Ok(val)
        }
    }
}
