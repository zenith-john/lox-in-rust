#[derive(Debug)]
pub struct ScanError {
    line: i32,
    reason: String,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scanner Error:\nLine {}: {}", self.line, self.reason)
    }
}
impl std::error::Error for ScanError {}

impl ScanError {
    pub fn new(line: i32, reason: String) -> ScanError {
        ScanError { line, reason }
    }
}

#[derive(Debug)]
pub struct ParseError {
    line: i32,
    reason: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser Error:\nLine {}: {}", self.line, self.reason)
    }
}
impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(line: i32, reason: String) -> ParseError {
        ParseError { line, reason }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    reason: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Runtime Error:\n{}", self.reason)
    }
}
impl std::error::Error for RuntimeError {}

impl RuntimeError {
    pub fn new(reason: String) -> RuntimeError {
        RuntimeError { reason }
    }
}
