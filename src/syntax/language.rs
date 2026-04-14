#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    Java,
    C,
    Bash,
    Plain,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" | "jsx" | "ts" | "tsx" => Language::JavaScript,
            "java" => Language::Java,
            "c" | "h" | "cpp" | "hpp" | "cc" => Language::C,
            "sh" | "bash" => Language::Bash,
            _ => Language::Plain,
        }
    }

    pub fn keywords(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while",
                "loop", "break", "continue", "return", "struct", "enum", "trait", "impl", "pub",
                "use", "mod", "crate", "self", "super", "as", "move", "ref", "unsafe", "async",
                "await", "dyn", "where", "type", "in",
            ],
            Language::Python => &[
                "def", "class", "if", "elif", "else", "for", "while", "break", "continue",
                "return", "try", "except", "finally", "with", "as", "import", "from", "pass",
                "raise", "assert", "lambda", "yield", "async", "await", "global", "nonlocal",
                "True", "False", "None", "and", "or", "not", "in", "is",
            ],
            Language::JavaScript => &[
                "function", "const", "let", "var", "if", "else", "for", "while", "break",
                "continue", "return", "class", "extends", "super", "this", "new", "try", "catch",
                "finally", "throw", "async", "await", "import", "export", "from", "default",
                "switch", "case", "typeof", "instanceof", "delete", "void", "yield",
            ],
            Language::Java => &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "goto", "if", "implements", "import", "instanceof",
                "int", "interface", "long", "native", "new", "package", "private", "protected",
                "public", "return", "short", "static", "strictfp", "super", "switch", "synchronized",
                "this", "throw", "throws", "transient", "try", "void", "volatile", "while",
                "true", "false", "null",
            ],
            Language::C => &[
                "int", "char", "float", "double", "void", "struct", "union", "enum", "if",
                "else", "for", "while", "do", "break", "continue", "return", "switch", "case",
                "default", "sizeof", "typedef", "static", "const", "extern", "auto", "register",
                "volatile", "unsigned", "signed", "long", "short",
            ],
            Language::Bash => &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case",
                "esac", "function", "return", "exit", "break", "continue", "local", "export",
                "source", "alias", "echo", "read", "test",
            ],
            Language::Plain => &[],
        }
    }

    pub fn types(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Cell", "RefCell",
            ],
            Language::Java => &[
                "byte", "short", "int", "long", "float", "double", "boolean", "char", "String",
                "Integer", "Double", "List", "ArrayList", "Map", "HashMap", "Set", "HashSet",
            ],
            Language::C => &["int", "char", "float", "double", "void", "size_t", "uint8_t", "uint16_t", "uint32_t"],
            _ => &[],
        }
    }
}
