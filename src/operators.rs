use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
	Add,
	Sub,
	Mul,
	Div,
	Exp,
	Eq
}

impl Operator {
	pub fn priority(&self) -> u8 {
		match self {
			Self::Eq => 20,
			Self::Add => 30,
			Self::Sub => 30,
			Self::Mul => 40,
			Self::Div => 40,
			Self::Exp => 50,
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
			"==" => Ok(Self::Eq),
			_ => Err(())
		}
	}
}
