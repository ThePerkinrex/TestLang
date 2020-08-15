use termcolor::{Buffer, ColorSpec, WriteColor};

use std::io::{Write, Result};

pub fn color<T: std::fmt::Display>(buf: &mut Buffer, txt: T, color: &ColorSpec) -> Result<()>{
	buf.set_color(color)?;
	write!(buf, "{}", txt)?;
	Ok(())
}

pub fn colorln<T: std::fmt::Display>(buf: &mut Buffer, txt: T, color: &ColorSpec) -> Result<()>{
	buf.set_color(color)?;
	writeln!(buf, "{}", txt)?;
	Ok(())
}