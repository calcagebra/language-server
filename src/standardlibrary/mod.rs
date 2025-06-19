use crate::types::NumberType;

// TODO: Add operands in this
/// Returns argument type and return type
pub fn internal_type_map(f: &str) -> (Vec<Vec<NumberType>>, NumberType) {
    match f {
        "read" => (vec![], NumberType::Real),
        "real" => (vec![vec![NumberType::Int]], NumberType::Real),
        "int" => (vec![vec![NumberType::Real]], NumberType::Int),
        "print" | "round" | "ceil" | "floor" | "ln" | "log10" | "sin" | "cos" | "tan" | "sqrt"
        | "cbrt" | "graph" => (vec![vec![NumberType::Real]], NumberType::Real),
        "log" | "nrt" => (
            vec![vec![NumberType::Real], vec![NumberType::Real]],
            NumberType::Real,
        ),
        "transpose" | "determinant" | "adj" | "inverse" => {
            (vec![vec![NumberType::Matrix]], NumberType::Matrix)
        }
        "abs" => (
            vec![vec![
                NumberType::Int,
                NumberType::Real,
                NumberType::Complex,
                NumberType::Matrix,
            ]],
            NumberType::Real,
        ),
        _ => (vec![vec![NumberType::Unknown]], NumberType::Unknown),
    }
}

pub const STD: [&str; 34] = [
    "print",
    "read",
    "int",
    "real",
    "add",
    "sub",
    "mul",
    "div",
    "pow",
    "rem",
    "is_eq",
    "neq",
    "gt",
    "gteq",
    "lt",
    "lteq",
    "abs",
    "round",
    "ceil",
    "floor",
    "ln",
    "log10",
    "log",
    "sin",
    "cos",
    "tan",
    "sqrt",
    "cbrt",
    "nrt",
    "graph",
    "transpose",
    "determinant",
    "adj",
    "inverse",
];
