use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StandardLibrary<'a> {
    pub map: HashMap<&'a str, &'a str>,
}

impl<'a> StandardLibrary<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn init_std(&mut self) -> &mut Self {
        self.map.insert(
            "print",
            "Prints numbers to stdout, numbers are always followed by a newline, returns 0.",
        );
        self.map.insert(
            "read",
            "Reads a number from stdin with the prompt `Enter number: ` and returns it.",
        );

        self.map.insert("round", "Returns the number rounded.");
        self.map.insert("ceil", "Returns the number ceiled.");
        self.map.insert("floor", "Returns the number floored.");

        self.map.insert("log", "Returns natural log of number.");

        self.map.insert("sin", "Returns sin of the radian.");
        self.map.insert("cos", "Returns cos of the radian.");
        self.map.insert("tan", "Returns tan of the radian.");

        self.map.insert("sqrt", "Returns square root of a number.");
        self.map.insert("cbrt", "Returns cube root of a number.");
        self.map.insert("nrt", "Returns nth root of a number.");

        self.map.insert("len", "Returns length of a set.");

        self.map.insert("get", "Get an element of a set.");
        self.map.insert("set", "Set an element of a set.");
        self.map.insert("sum", "Returns the sum of elements in a set.");
        self.map.insert("product", "Returns the product of the elements in a set.");

        self.map.insert("map", "Map a function onto a set.");

        self.map.insert("graph", "Plot the graph of a function to std.");

        self
    }
}
