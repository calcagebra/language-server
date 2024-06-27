mod ast;
mod errors;
mod lexer;
mod parser;
mod standardlibrary;
mod token;

use ast::Ast;
use dashmap::DashMap;

use lexer::Lexer;
use parser::Parser;
use serde_json::Value;
use simsearch::{SearchOptions, SimSearch};
use standardlibrary::StandardLibrary;
use token::Token;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone)]
struct Backend {
    client: Client,
    file: DashMap<usize, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "calcagebra-ls initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, param: DidOpenTextDocumentParams) {
        self.file.clear();
        let lines = param
            .text_document
            .text
            .lines()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        for (i, line) in lines.iter().enumerate() {
            self.file.insert(i, line.to_string());
        }

        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, param: DidChangeTextDocumentParams) {
        self.file.clear();
        let lines = param.content_changes[0]
            .text
            .lines()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        for (i, line) in lines.iter().enumerate() {
            self.file.insert(i, line.to_string());
        }

        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, param: CompletionParams) -> Result<Option<CompletionResponse>> {
        let file = &self
            .file
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let mut variables = vec!["π".to_string(), "pi".to_string(), "e".to_string()];
        let mut functions = vec![];
        let tokens = Token::dictionary();
        let mut std = StandardLibrary::new();
        std.init_std();
        let std_functions = std
            .map
            .keys()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let lex = Lexer::new(file).tokens();
        Parser::new(lex)
            .ast()
            .iter()
            .filter(|f| matches!(f, Ast::Assignment(_, _) | Ast::FunctionDeclaration(_, _, _)))
            .for_each(|f| match f {
                Ast::Assignment(ident, _) => variables.push(ident.to_string()),
                Ast::FunctionDeclaration(name, _, _) => functions.push(name.to_string()),
                _ => unreachable!(),
            });

        let line = param.text_document_position.position.line as usize;
        let character = param.text_document_position.position.character as usize;
        let line = self.file.get(&line).unwrap().to_string();
        let mut text = String::new();
        for (i, char) in line.split("").enumerate() {
            let c = char.chars().next();
            if c.is_some() && !c.unwrap().is_ascii_alphabetic() {
                text = String::new()
            }
            text += char;
            if i == character {
                break;
            }
        }

        let mut responses = vec![];

        self.get_closest_match(&text, variables)
            .iter()
            .for_each(|f| {
                responses.push(CompletionItem {
                    label: f.to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    ..Default::default()
                })
            });

        self.get_closest_match(&text, functions)
            .iter()
            .for_each(|f| {
                responses.push(CompletionItem {
                    label: f.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                })
            });

        self.get_closest_match(&text, tokens).iter().for_each(|f| {
            responses.push(CompletionItem {
                label: f.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            })
        });

        self.get_closest_match(&text, std_functions)
            .iter()
            .for_each(|f| {
                responses.push(CompletionItem {
                    label: f.to_string(),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: std.map.get(f.to_string().as_str()).unwrap().to_string(),
                    })),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                })
            });

        Ok(Some(CompletionResponse::Array(responses)))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Array(vec![]),
            range: None,
        }))
    }
}

impl Backend {
    pub fn get_closest_match(&self, word: &str, words: Vec<String>) -> Vec<String> {
        let engine_options = SearchOptions::new().threshold(0.55);
        let mut engine: SimSearch<u32> = SimSearch::new_with(engine_options);

        for (id, token) in words.iter().enumerate() {
            engine.insert(id as u32, token);
        }

        let ids = engine.search(word);
        let mut matches = vec![];

        for id in ids {
            matches.push(words.get(id as usize).unwrap().to_string());
        }

        matches
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        file: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
