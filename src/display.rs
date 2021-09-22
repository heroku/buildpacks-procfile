//! Display output to users and capture it in a thread local buffer for testing
//!
//! Logs are captured by a thread local buffer so they can be asserted
//! against in test.
//!
//! Log output format matches Heroku style
//!
//! Example:
//!
//!```
//! use display;
//!
//! display::is_test(true);
//! display::header("I am a header");
//!
//! display::assert_contains("[I am a header]");
//!
//! let out = display::as_str();
//! assert! out.contains("[I am a header]");
//!```

use std::cell::RefCell;
use std::io::Write;
use termcolor::{Buffer, Color, ColorSpec, StandardStream, WriteColor};

thread_local! {
    static THREAD_LOCAL_USER_OUTPUT: RefCell<StdoutOrBufferOutput> = RefCell::new(StdoutOrBufferOutput::new());
}

#[allow(dead_code)]
pub fn with_link(message: impl AsRef<str>, link: impl AsRef<str>) -> String {
    format!("{}\n\nLink: {}", message.as_ref(), link.as_ref())
}

#[allow(dead_code)]
pub fn error(header: impl AsRef<str>, body: impl AsRef<str>) {
    let mut header_color = ColorSpec::new();
    let mut body_color = ColorSpec::new();

    header_color.set_fg(Some(Color::Red)).set_bold(true);
    body_color.set_fg(Some(Color::Red));

    write_with_color(format!("\n[Error: {}]", header.as_ref()), &header_color);
    write_with_color(body.as_ref(), &body_color);
}

#[allow(dead_code)]
pub fn warning(header: impl AsRef<str>, body: impl AsRef<str>) {
    let mut header_color = ColorSpec::new();
    let mut body_color = ColorSpec::new();

    header_color.set_fg(Some(Color::Yellow)).set_bold(true);
    body_color.set_fg(Some(Color::Yellow));

    write_with_color(format!("\n[Warning: {}]", header.as_ref()), &header_color);
    write_with_color(body.as_ref(), &body_color);
}

#[allow(dead_code)]
pub fn header(header: impl AsRef<str>) {
    let mut header_color = ColorSpec::new();
    header_color.set_fg(Some(Color::Magenta)).set_bold(true);

    write_with_color(format!("\n[{}]", header.as_ref()), &header_color);
}

#[allow(dead_code)]
pub fn info(message: impl AsRef<str>) {
    THREAD_LOCAL_USER_OUTPUT.with(|log_ref| {
        let user_out_with_capture = &mut *log_ref.borrow_mut();

        user_out_with_capture.writeln(message.as_ref());
        user_out_with_capture.flush();
    });
}

#[allow(dead_code)]
pub fn is_test(test: bool) {
    THREAD_LOCAL_USER_OUTPUT.with(|log_ref| {
        let user_out_with_capture = &mut *log_ref.borrow_mut();
        user_out_with_capture.is_test(test);
    });
}

#[allow(dead_code)]
pub fn as_str() -> String {
    THREAD_LOCAL_USER_OUTPUT.with(|log_ref| {
        let user_out_with_capture = &mut *log_ref.borrow_mut();
        user_out_with_capture.as_str()
    })
}

#[allow(dead_code)]
pub fn assert_contains(substring: impl AsRef<str>) {
    let body = as_str();
    let substring = substring.as_ref();
    assert!(
        body.contains(substring),
        "Expected log to contain '{}' but it did not.\nLog contents:\n{}",
        substring,
        body,
    );
}

fn write_with_color(message: impl AsRef<str>, color: &ColorSpec) {
    THREAD_LOCAL_USER_OUTPUT.with(|log_ref| {
        let user_out_with_capture = &mut *log_ref.borrow_mut();

        user_out_with_capture.set_color(color);
        user_out_with_capture.writeln(message);
        user_out_with_capture.reset();
        user_out_with_capture.flush();
    });
}

struct StdoutOrBufferOutput {
    stream: Box<dyn StringifyWithColor>,
}
trait StringifyWithColor: WriteColor {
    fn stringify(&self) -> String;
}
impl StringifyWithColor for Buffer {
    fn stringify(&self) -> String {
        std::str::from_utf8(self.as_slice()).unwrap().to_string()
    }
}
impl StringifyWithColor for StandardStream {
    fn stringify(&self) -> String {
        panic!("Display test mode is disabled. Enable test mode to capture logs `display::is_test(true)`")
    }
}

// Handles switching between stdout and a buffer
impl StdoutOrBufferOutput {
    pub fn new() -> Self {
        StdoutOrBufferOutput {
            stream: Box::new(StandardStream::stdout(termcolor::ColorChoice::Always)),
        }
    }

    pub fn is_test(&mut self, test: bool) {
        if test {
            self.stream = Box::new(Buffer::ansi());
        } else {
            self.stream = Box::new(StandardStream::stdout(termcolor::ColorChoice::Always));
        }
    }

    pub fn writeln(&mut self, message: impl AsRef<str>) {
        writeln!(&mut self.stream, "{}", message.as_ref()).unwrap();
    }

    pub fn set_color(&mut self, spec: &ColorSpec) {
        self.stream.set_color(spec).unwrap();
    }

    pub fn reset(&mut self) {
        self.stream.reset().unwrap();
    }

    pub fn flush(&mut self) {
        self.stream.flush().unwrap();
    }

    #[allow(dead_code)]
    pub fn as_str(&mut self) -> String {
        self.stream.stringify()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        is_test(true);
        error("header", "body");

        assert_contains("[Error: header]");
        assert_contains("body");
    }

    #[test]
    fn test_warning() {
        is_test(true);
        warning("header", "body");

        assert_contains("[Warning: header]");
        assert_contains("body");
    }

    #[test]
    fn test_header() {
        is_test(true);
        header("header");

        assert_contains("[header]");
    }

    #[test]
    fn test_info() {
        is_test(true);
        info("info");

        assert_contains("info");

        // Make sure not other tests are leaking through
        assert_eq!("info\n", as_str());
    }

    #[test]
    fn test_with_link() {
        is_test(true);
        info(with_link("info", "https://www.heroku.com"));

        assert_contains("info");
        assert_contains("Link: https://www.heroku.com");
    }
}
