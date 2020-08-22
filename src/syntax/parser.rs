use crate::lexer::{Lexer, Lexeme, Class};
use super::trees::*;

pub trait TryMaybeParse: Sized {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>>;
	fn try_parse(parser: &mut Parser) -> ParseResult<Self> {
		Self::try_maybe_parse(parser).transpose().unwrap_or(Err(ParseError::Invalid))
	}
}

pub trait MaybeParse: Sized {
	fn maybe_parse(parser: &mut Parser) -> Option<Self>;
	fn try_parse(parser: &mut Parser) -> ParseResult<Self> {
		Self::maybe_parse(parser).ok_or(ParseError::Invalid)
	}
}

pub trait Parse: Sized {
	fn parse(parser: &mut Parser) -> Self;
}

pub trait TryParse: Sized {
	fn try_parse(parser: &mut Parser) -> ParseResult<Self>;
}

pub struct Parser<'source> {
	pub lexer: Lexer<'source>,
	pub lexeme: Lexeme,
}

impl<'source> Parser<'source> {
	pub fn new(mut lexer: Lexer<'source>) -> Self {
		Self {
			lexeme: lexer.lex(),
			lexer,
		}
	}

	fn step(&mut self) {
		self.lexeme = self.lexer.lex();
	}
}

impl MaybeParse for Quote {
	fn maybe_parse(parser: &mut Parser) -> Option<Self> {
		Some(match &parser.lexeme.class.clone() {
			Class::Quote(quote) => {
				parser.step();
				Self {
					quote: quote.clone(),
				}
			},	
			_ => return None,
		})
	}
}

impl MaybeParse for Tag {
	fn maybe_parse(parser: &mut Parser) -> Option<Self> {
		Some(match parser.lexeme.class.clone() {
			Class::Tag(tag) => {
				parser.step();
				Self {
					tag: tag.clone(),
				}
			},
			_ => return None,
		})
	}
}

impl Parse for Extent {
	fn parse(parser: &mut Parser) -> Self {
		match parser.lexeme.class {
			Class::Dynamic => {
				parser.step();
				Extent::Nonstatic
			},
			Class::Static => {
				parser.step();
				Extent::Nondynamic
			},
			_ => return Extent::Universal,
		}
	}
}

impl TryMaybeParse for Abstraction {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>> {
		if parser.lexeme.class != Class::Abstract {
			return Ok(None);
		}
		parser.step();

		if parser.lexeme.class != Class::LeftParen {
			return Err(ParseError::Expected("LeftParen in abstraction"));
		}
		parser.step();

		let mut sequence = Vec::new();
		if parser.lexeme.class != Class::RightParen {
			loop {
				let statement = Statement::try_parse(parser)?;
				sequence.push(statement);
				match parser.lexeme.class {
					Class::RightParen => break,
					Class::Semicolon => continue,
					_ => return Err(ParseError::Expected("RightParen or SemiColon following statement in abstraction"))
				}
			}
		}
		parser.step();
		Ok(Some(Self {
			sequence
		}))
	}
}

impl TryMaybeParse for ExponentialType {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>> {
		if parser.lexeme.class != Class::LeftBrace {
			return Ok(None);
		}
		parser.step();

		let domain = Expr::try_parse(parser)?;
		if parser.lexeme.class != Class::IntoLazy {
			return Err(ParseError::Expected("IntoLazy in exponential"));
		}
		parser.step();
		
		let codomain = Expr::try_parse(parser)?;
		if parser.lexeme.class != Class::RightBrace {
			return Err(ParseError::Expected("RightBrace in exponential"));
		}
		parser.step();

		Ok(Some(Self {
			domain,
			codomain,
		}))
	}
}

impl TryMaybeParse for OrdinalType {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>> {
		if parser.lexeme.class != Class::LeftAngle {
			return Ok(None);
		}
		parser.step();
		
		let mut labels = Vec::new();
		if parser.lexeme.class != Class::RightAngle {
			loop {
				let tag = Tag::try_parse(parser)?;
				labels.push(tag);
				match parser.lexeme.class {
					Class::RightAngle => break,
					Class::Comma => continue,
					_ => return Err(ParseError::Expected("RightAngle or Comma following tag in ordinal"))
				}
			}
		}
		parser.step();
		Ok(Some(Self {
			labels,
		}))
	}
}

impl TryMaybeParse for ClosedExpr {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>> {
		Ok(Some(if let Some(quote) = Quote::maybe_parse(parser) {
			ClosedExpr::Quote(quote)
		} else if let Some(tag) = Tag::maybe_parse(parser) {
			ClosedExpr::Tag(tag)
		} else if let Some(abstraction) = Abstraction::try_maybe_parse(parser).transpose() {
			ClosedExpr::Abstraction(Box::new(abstraction?))
		} else if let Some(exponential) = ExponentialType::try_maybe_parse(parser).transpose() {
			ClosedExpr::ExponentialType(Box::new(exponential?))
		} else if let Some(ordinal) = OrdinalType::try_maybe_parse(parser).transpose() {
			ClosedExpr::OrdinalType(ordinal?)
		} else {
			return Ok(None);
		}))
	}
}

impl TryParse for Expr {
	fn try_parse(parser: &mut Parser) -> ParseResult<Self> {
		let closed = ClosedExpr::try_parse(parser)?;
		if parser.lexeme.class == Class::Apply {
			parser.step();
			let argument = ClosedExpr::try_parse(parser)?;
			return Ok(Expr::Application{
				operator: closed,
				argument,
			})
		}
		Ok(Expr::ClosedExpr(closed))
	}
}

impl TryMaybeParse for Definition {
	fn try_maybe_parse(parser: &mut Parser) -> ParseResult<Option<Self>> {
		if parser.lexeme.class != Class::Define {
			return Ok(None);
		}
		parser.step();

		let extent = Extent::parse(parser);

		let tag = Tag::try_parse(parser)?;

		if parser.lexeme.class != Class::Typify {
			return Err(ParseError::Expected("Typify in definition"));
		}
		parser.step();

		let r#type = Expr::try_parse(parser)?;

		if parser.lexeme.class != Class::Equal {
			return Err(ParseError::Expected("Equal in definition"));
		}
		parser.step();

		let value = Expr::try_parse(parser)?;

		Ok(Some(Self {
			extent,
			tag,
			r#type,
			value,
		}))
	}
}

impl TryParse for Statement {
	fn try_parse(parser: &mut Parser) -> ParseResult<Self> {
		Ok(if let Some(definition) = Definition::try_maybe_parse(parser).transpose() {
			Statement::Definition(definition?)
		} else {
			Statement::Expr(Expr::try_parse(parser)?)
		})
	}
}

impl TryParse for File {
	fn try_parse(parser: &mut Parser) -> ParseResult<Self> {
		let mut definitions = Vec::new();
		loop {
			if Class::Sentinel == parser.lexeme.class {
				break;
			} else if let Some(definition) = Definition::try_maybe_parse(parser).transpose() {
				definitions.push(definition?);
			} else {
				return Err(ParseError::Expected("Definition or EOF in file"));
			}
		}
		Ok(Self {
			definitions
		})
	}
}

#[derive(Debug)]
pub enum ParseError {
	Invalid,
	Expected(&'static str),
}

pub type ParseResult<T> = Result<T, ParseError>;