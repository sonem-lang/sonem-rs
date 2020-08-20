use ascii::AsciiString;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Class {
	Define,
	IntoLazy,
	Tag(AsciiString),
	Quote(Vec<u8>),
	Typify,
	Dynamic,
	Abstract,
	Equal,
	Apply,
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	LeftAngle,
	RightAngle,
	Comma,
	Semicolon,
	Invalid,
	Sentinel,
}

#[derive(Debug)]
pub struct Lexeme {
	pub class: Class,
	pub location: usize,
}

impl<'source> Lexeme {
	pub fn class(&self) -> Class {
		self.class.clone()
	}
}

pub struct Lexer<'source> {
	source: &'source [u8],
	cursor: usize,
}

impl<'source> Lexer<'source> {
	pub fn new (source: &'source [u8]) -> Option<Self> {
		if source.len() == usize::MAX {
			None
		} else {
			Some(Self {
				source,
				cursor: 0,
			})
		}
	}

	fn peek(&self) -> Option<u8> {
		self.source.get(self.cursor).cloned()
	}

	fn next(&mut self) {
		self.cursor += 1;
	}

	fn lex_tag(&mut self) -> Class {
		let location = self.cursor - 1;
		loop {
			if let Some(subsequent) = self.peek() {
				match subsequent {
					b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => self.next(),
					_ => break,
				}
			} else {
				break;
			}
		}
		Class::Tag(
			unsafe {
				AsciiString::from_ascii_unchecked(&self.source[location..self.cursor])
			}
		)
	}

	fn lex_quote(&mut self) -> Class {
		let location = self.cursor - 1;
		loop {
			if let Some(subsequent) = self.peek() {
				self.next();
				match subsequent {
					b'"' => break,
					b'\\' => {
						if let Some(_) = self.peek() {
							continue;
						} else {
							return Class::Invalid;
						}
					}
					_ => continue,
				}
			} else {
				return Class::Invalid;
			}
		}
		Class::Quote(Vec::from(&self.source[location + 1..self.cursor - 1]))
	}

	fn lex_prehyphenated(&mut self) -> Class {
		match self.peek() {
			Some(b'>') => {
				self.next();
				Class::IntoLazy
			},
			_ => Class::Invalid,
		}
	}

	pub fn lex(&mut self) -> Lexeme {
		'restart: loop { 
			let location = self.cursor;
			if let Some(initial) = self.peek() {
				self.next();
				use Class::*;
				return Lexeme {
					class: match initial {
						b' ' | b'\n' | b'\t' => {
							continue 'restart;
						}
						b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => self.lex_tag(),
						b'"' => self.lex_quote(),
						b'-' => self.lex_prehyphenated(),
						b'|' => Define,
						b'~' => Dynamic,
						b':' => Typify,
						b'=' => Equal,
						b'$' => Abstract,
						b'.' => Apply,
						b'(' => LeftParen,
						b')' => RightParen,
						b'{' => LeftBrace,
						b'}' => RightBrace,
						b'<' => LeftAngle,
						b'>' => RightAngle,
						b',' => Comma,
						b';' => Semicolon,
						_ => Invalid,
					},
					location,
				};
			} else {
				return Lexeme {
					class: Class::Sentinel,
					location
				};
			}
		}
	}
}