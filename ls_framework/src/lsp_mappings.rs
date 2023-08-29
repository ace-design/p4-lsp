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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum HighlightType {
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

impl HighlightType {
    pub fn get(&self) -> lsp_types::SemanticTokenType {
        match self {
            HighlightType::Namespace => lsp_types::SemanticTokenType::NAMESPACE,
            HighlightType::Type => lsp_types::SemanticTokenType::TYPE,
            HighlightType::Class => lsp_types::SemanticTokenType::CLASS,
            HighlightType::Enum => lsp_types::SemanticTokenType::ENUM,
            HighlightType::Interface => lsp_types::SemanticTokenType::INTERFACE,
            HighlightType::Struct => lsp_types::SemanticTokenType::STRUCT,
            HighlightType::TypeParameter => lsp_types::SemanticTokenType::TYPE_PARAMETER,
            HighlightType::Parameter => lsp_types::SemanticTokenType::PARAMETER,
            HighlightType::Variable => lsp_types::SemanticTokenType::VARIABLE,
            HighlightType::Property => lsp_types::SemanticTokenType::PROPERTY,
            HighlightType::EnumMember => lsp_types::SemanticTokenType::ENUM_MEMBER,
            HighlightType::Event => lsp_types::SemanticTokenType::EVENT,
            HighlightType::Function => lsp_types::SemanticTokenType::FUNCTION,
            HighlightType::Method => lsp_types::SemanticTokenType::METHOD,
            HighlightType::Macro => lsp_types::SemanticTokenType::MACRO,
            HighlightType::Keyword => lsp_types::SemanticTokenType::KEYWORD,
            HighlightType::Modifier => lsp_types::SemanticTokenType::MODIFIER,
            HighlightType::Comment => lsp_types::SemanticTokenType::COMMENT,
            HighlightType::String => lsp_types::SemanticTokenType::STRING,
            HighlightType::Number => lsp_types::SemanticTokenType::NUMBER,
            HighlightType::Regexp => lsp_types::SemanticTokenType::REGEXP,
            HighlightType::Operator => lsp_types::SemanticTokenType::OPERATOR,
            HighlightType::Decorator => lsp_types::SemanticTokenType::DECORATOR,
        }
    }
}
