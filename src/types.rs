use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum NumberType {
	Int,
	Real,
	Complex,
	Matrix,
	Unknown,
}

impl NumberType {
	pub fn parse(ident: &str) -> Self {
		match ident.to_uppercase().as_str() {
			"Z" | "INT" | "INTEGER" => Self::Int,
			"R" | "FLOAT" => Self::Real,
			"C" | "COMPLEX" => Self::Complex,
			"MATRIX" => Self::Matrix,
			_ => unimplemented!(),
		}
	}
}

impl Display for NumberType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				NumberType::Int => "Z",
				NumberType::Real => "R",
				NumberType::Complex => "C",
				NumberType::Matrix => "Matrix",
				NumberType::Unknown => "Unknown"
			}
		)
	}
}
