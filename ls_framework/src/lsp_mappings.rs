use serde::Deserialize;
use tower_lsp::lsp_types::{self, CompletionItemKind};

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
pub enum SemanticTokenType {
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

impl SemanticTokenType {
    pub fn get(&self) -> lsp_types::SemanticTokenType {
        match self {
            SemanticTokenType::Namespace => lsp_types::SemanticTokenType::NAMESPACE,
            SemanticTokenType::Type => lsp_types::SemanticTokenType::TYPE,
            SemanticTokenType::Class => lsp_types::SemanticTokenType::CLASS,
            SemanticTokenType::Enum => lsp_types::SemanticTokenType::ENUM,
            SemanticTokenType::Interface => lsp_types::SemanticTokenType::INTERFACE,
            SemanticTokenType::Struct => lsp_types::SemanticTokenType::STRUCT,
            SemanticTokenType::TypeParameter => lsp_types::SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::Parameter => lsp_types::SemanticTokenType::PARAMETER,
            SemanticTokenType::Variable => lsp_types::SemanticTokenType::VARIABLE,
            SemanticTokenType::Property => lsp_types::SemanticTokenType::PROPERTY,
            SemanticTokenType::EnumMember => lsp_types::SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::Event => lsp_types::SemanticTokenType::EVENT,
            SemanticTokenType::Function => lsp_types::SemanticTokenType::FUNCTION,
            SemanticTokenType::Method => lsp_types::SemanticTokenType::METHOD,
            SemanticTokenType::Macro => lsp_types::SemanticTokenType::MACRO,
            SemanticTokenType::Keyword => lsp_types::SemanticTokenType::KEYWORD,
            SemanticTokenType::Modifier => lsp_types::SemanticTokenType::MODIFIER,
            SemanticTokenType::Comment => lsp_types::SemanticTokenType::COMMENT,
            SemanticTokenType::String => lsp_types::SemanticTokenType::STRING,
            SemanticTokenType::Number => lsp_types::SemanticTokenType::NUMBER,
            SemanticTokenType::Regexp => lsp_types::SemanticTokenType::REGEXP,
            SemanticTokenType::Operator => lsp_types::SemanticTokenType::OPERATOR,
            SemanticTokenType::Decorator => lsp_types::SemanticTokenType::DECORATOR,
        }
    }
}
