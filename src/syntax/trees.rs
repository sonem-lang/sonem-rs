use ascii::AsciiString;
//"xxx"
#[derive(Debug)]
pub struct Quote {
	pub quote: Vec<u8>,
}

//abc_123
#[derive(Debug)]
pub struct Tag {
	pub tag: AsciiString,
}

// ~
#[derive(Debug)]
pub enum Phase {
	Dynamic,
}

//$ (first; second; last) or //$ {dom -> cod} (...)
#[derive(Debug)]
pub struct Abstraction {
	pub sequence: Vec<Statement>,
}

// {dom -> cod}
#[derive(Debug)]
pub struct ExponentialType {
	pub domain: Expr,
	pub codomain: Expr,
}

// <a, b, c>
#[derive(Debug)]
pub struct OrdinalType {
	pub labels: Vec<Tag>,
}

#[derive(Debug)]
pub enum ClosedExpr {
	Quote(Quote),
	Tag(Tag),
	Abstraction(Box<Abstraction>),
	ExponentialType(Box<ExponentialType>),
	OrdinalType(OrdinalType),
}

// see above
#[derive(Debug)]
pub enum Expr {
	//fun.ex or fun.(ex, why)
	Application {
		operator: ClosedExpr,
		argument: ClosedExpr,
	},
	ClosedExpr(ClosedExpr),
}

// | optional_phase tag: OptionalTypeExpr = value_expr
#[derive(Debug)]
pub struct Definition {
	pub phase: Phase,
	pub tag: Tag,
	pub r#type: Expr,
	pub value: Expr,
}

// see above
#[derive(Debug)]
pub enum Statement {
	Definition(Definition),
	Expr(Expr),
}

// defn; defn2; defn3
#[derive(Debug)]
pub struct File {
	pub definitions: Vec<Definition>,
}