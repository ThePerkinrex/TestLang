use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
	Add,
	Sub,
	Mul,
	Div,
	Exp,
}

impl Operator {
	pub fn priority(&self) -> u8 {
		match self {
			Self::Add => 0,
			Self::Sub => 0,
			Self::Mul => 1,
			Self::Div => 1,
			Self::Exp => 2,
		}
	}
}

impl TryFrom<&str> for Operator {
	type Error = ();
	fn try_from(c: &str) -> Result<Self, <Self as std::convert::TryFrom<&str>>::Error> {
		match c {
			"+" => Ok(Self::Add),
			"-" => Ok(Self::Sub),
			"*" => Ok(Self::Mul),
			"/" => Ok(Self::Div),
			"**" => Ok(Self::Exp),
			_ => Err(())
		}
	}
}
