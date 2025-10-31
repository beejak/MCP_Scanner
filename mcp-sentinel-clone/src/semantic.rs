use tree_sitter::{Parser, Language};

extern "C" {
    fn tree_sitter_python() -> Language;
    fn tree_sitter_javascript() -> Language;
}

pub enum SupportedLanguage {
    Python,
    JavaScript,
}

pub fn a_parser_for(language: SupportedLanguage) -> Parser {
    let mut parser = Parser::new();
    let language = match language {
        SupportedLanguage::Python => unsafe { tree_sitter_python() },
        SupportedLanguage::JavaScript => unsafe { tree_sitter_javascript() },
    };
    parser.set_language(language).unwrap();
    parser
}
