use serde::Deserialize;
use tower_lsp::lsp_types::{CompletionItemKind, SemanticTokenType};

#[derive(Debug, Deserialize, Clone)]
pub enum SymbolCompletionType {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl SymbolCompletionType {
    pub fn get(&self) -> CompletionItemKind {
        match self {
            SymbolCompletionType::Text => CompletionItemKind::TEXT,
            SymbolCompletionType::Method => CompletionItemKind::METHOD,
            SymbolCompletionType::Function => CompletionItemKind::FUNCTION,
            SymbolCompletionType::Constructor => CompletionItemKind::CONSTRUCTOR,
            SymbolCompletionType::Field => CompletionItemKind::FIELD,
            SymbolCompletionType::Variable => CompletionItemKind::VARIABLE,
            SymbolCompletionType::Class => CompletionItemKind::CLASS,
            SymbolCompletionType::Interface => CompletionItemKind::INTERFACE,
            SymbolCompletionType::Module => CompletionItemKind::MODULE,
            SymbolCompletionType::Property => CompletionItemKind::PROPERTY,
            SymbolCompletionType::Unit => CompletionItemKind::UNIT,
            SymbolCompletionType::Value => CompletionItemKind::VALUE,
            SymbolCompletionType::Enum => CompletionItemKind::ENUM,
            SymbolCompletionType::Keyword => CompletionItemKind::KEYWORD,
            SymbolCompletionType::Snippet => CompletionItemKind::SNIPPET,
            SymbolCompletionType::Color => CompletionItemKind::COLOR,
            SymbolCompletionType::File => CompletionItemKind::FILE,
            SymbolCompletionType::Reference => CompletionItemKind::REFERENCE,
            SymbolCompletionType::Folder => CompletionItemKind::FOLDER,
            SymbolCompletionType::EnumMember => CompletionItemKind::ENUM_MEMBER,
            SymbolCompletionType::Constant => CompletionItemKind::CONSTANT,
            SymbolCompletionType::Struct => CompletionItemKind::STRUCT,
            SymbolCompletionType::Event => CompletionItemKind::EVENT,
            SymbolCompletionType::Operator => CompletionItemKind::OPERATOR,
            SymbolCompletionType::TypeParameter => CompletionItemKind::TYPE_PARAMETER,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
enum SemanticTokenTypes {
    Namespace,
    Type,
    Class,
    Enum,
    Interface,
    Struct,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Event,
    Function,
    Method,
    Macro,
    Keyword,
    Modifier,
    Comment,
    String,
    Number,
    Regexp,
    Operator,
    Decorator,
}

impl SemanticTokenTypes {
    pub fn get(&self) -> SemanticTokenType {
        match self {
            SemanticTokenTypes::Namespace => SemanticTokenType::NAMESPACE,
            SemanticTokenTypes::Type => SemanticTokenType::TYPE,
            SemanticTokenTypes::Class => SemanticTokenType::CLASS,
            SemanticTokenTypes::Enum => SemanticTokenType::ENUM,
            SemanticTokenTypes::Interface => SemanticTokenType::INTERFACE,
            SemanticTokenTypes::Struct => SemanticTokenType::STRUCT,
            SemanticTokenTypes::TypeParameter => SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenTypes::Parameter => SemanticTokenType::PARAMETER,
            SemanticTokenTypes::Variable => SemanticTokenType::VARIABLE,
            SemanticTokenTypes::Property => SemanticTokenType::PROPERTY,
            SemanticTokenTypes::EnumMember => SemanticTokenType::ENUM_MEMBER,
            SemanticTokenTypes::Event => SemanticTokenType::EVENT,
            SemanticTokenTypes::Function => SemanticTokenType::FUNCTION,
            SemanticTokenTypes::Method => SemanticTokenType::METHOD,
            SemanticTokenTypes::Macro => SemanticTokenType::MACRO,
            SemanticTokenTypes::Keyword => SemanticTokenType::KEYWORD,
            SemanticTokenTypes::Modifier => SemanticTokenType::MODIFIER,
            SemanticTokenTypes::Comment => SemanticTokenType::COMMENT,
            SemanticTokenTypes::String => SemanticTokenType::STRING,
            SemanticTokenTypes::Number => SemanticTokenType::NUMBER,
            SemanticTokenTypes::Regexp => SemanticTokenType::REGEXP,
            SemanticTokenTypes::Operator => SemanticTokenType::OPERATOR,
            SemanticTokenTypes::Decorator => SemanticTokenType::DECORATOR,
        }
    }
}
