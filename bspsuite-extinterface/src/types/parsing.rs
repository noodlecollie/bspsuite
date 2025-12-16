use std::ops::Range;

/// Helper type representing a result which could be a parsing error.
pub type ParseResult<T = ()> = Result<T, ParseError>;

/// Helper struct representing a parsing error.
#[derive(Debug)]
pub struct ParseError
{
	/// The input line on which the error was encountered.
	pub line: usize,

	/// The column in the input line at which the error was encountered.
	pub column: usize,

	/// The token that caused the error, or None if there was no token (eg. the
	/// end of the file was encountered).
	pub token: Option<String>,

	/// A human-readable description of the error.
	pub description: String,
}

impl ParseError
{
	/// Create a new parse error.
	pub fn new(
		line: usize,
		column: usize,
		token: Option<String>,
		description: Option<String>,
	) -> Self
	{
		return Self {
			line: line,
			column: column,
			token: token,
			description: description.unwrap_or_default(),
		};
	}
}

/// Helper struct for calculating line and column indices.
///
/// This is designed to be used in a lexer or parser with no backtracking,
/// such as [logos](https://docs.rs/logos/latest/logos/).
#[derive(Clone)]
pub struct LineCounter
{
	current_line: usize,
	begin_index: usize,
}

impl LineCounter
{
	/// Create a new line counter, with the current line as 1 and the index in
	/// the input source as 0.
	pub fn new() -> Self
	{
		return Self {
			current_line: 1,
			begin_index: 0,
		};
	}

	/// Update the line counter with the index of a newline character.
	/// This increments the current line, and sets the index of this line to the
	/// provided index.
	pub fn record_new_line(&mut self, current_index: usize)
	{
		self.current_line += 1;
		self.begin_index = current_index;
	}

	/// Get the line and column numbers for a token on the current line,
	/// represented by a span in the raw input data.
	///
	/// This function panics if the provided span is in any way less than the
	/// begin index for the current line. This would indicate a token on a
	/// previous line.
	pub fn get_line_and_column(&self, token_span: Range<usize>) -> (usize, usize)
	{
		assert!(
			token_span.start >= self.begin_index || token_span.end >= self.begin_index,
			"get_line_and_column() called for a token not on the current line"
		);

		return (self.current_line, token_span.start - self.begin_index);
	}
}

impl Default for LineCounter
{
	fn default() -> Self
	{
		return LineCounter::new();
	}
}
