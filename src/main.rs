mod ast;
mod lexer;
mod parser;
mod standardlibrary;
mod token;
mod types;

use std::f32::consts::{E, PI};

use ast::AstNode;
use dashmap::DashMap;

use lexer::Lexer;
use parser::Parser;
use serde_json::Value;
use simsearch::{SearchOptions, SimSearch};
use token::Token;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::standardlibrary::{STD, internal_type_map};
use crate::types::NumberType;

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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
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
                diagnostic_provider: Some(DiagnosticServerCapabilities::RegistrationOptions(
                    DiagnosticRegistrationOptions {
                        ..Default::default()
                    },
                )),
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

    async fn diagnostic(
        &self,
        _: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let mut file = self
            .file
            .iter()
            .map(|f| (*f.key(), f.to_string()))
            .collect::<Vec<(usize, String)>>();

        file.sort_by_key(|k| k.0);

        let file = &file
            .iter()
            .map(|(_, v)| v.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let char_per_line = &file.lines().map(|f| f.len() + 1).collect::<Vec<usize>>();

        let mut items = vec![];

        Parser::new(Lexer::new(file).tokens())
            .ast()
            .unwrap_or_default()
            .iter()
            .filter(|f| matches!(f, AstNode::Error(..)))
            .for_each(|f| {
                if let AstNode::Error(message, range) = f {
                    let (start, end) = range.clone().into_inner();

                    let mut start_line = 0;
                    let mut start_sum = 0;

                    let mut end_line = 0;
                    let mut end_sum = 0;

                    for chars in char_per_line {
                        start_sum += chars;
                        if start <= start_sum + 1 {
                            break;
                        }
                        start_line += 1;
                    }

                    for chars in char_per_line {
                        if end <= end_sum + 1 {
                            break;
                        }
                        end_sum += chars;
                        end_line += 1;
                    }

                    items.push(Diagnostic {
                        range: Range::new(
                            Position::new(
                                start_line,
                                // (start_sum as isize - start as isize)
                                //     .abs()
                                //     .try_into()
                                //     .unwrap(),
                                0,
                            ),
                            Position::new(
                                end_line,
                                // (end_sum as isize - end as isize).abs().try_into().unwrap(),
                                0,
                            ),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: message.to_string(),
                        source: Some("calcagebra".to_string()),
                        ..Default::default()
                    })
                }
            });

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
            }),
        ))
    }

    async fn completion(&self, param: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, "completion requested!")
            .await;

        let file = &self
            .file
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let mut variables = ["pi", "π", "e"]
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let mut functions = vec![];
        let functions_docs = DashMap::new();

        let tokens = Token::dictionary();

        Parser::new(Lexer::new(file).tokens())
            .ast()
            .unwrap_or_default()
            .iter()
            .filter(|f| {
                matches!(
                    f,
                    AstNode::Assignment(..) | AstNode::FunctionDeclaration(..)
                )
            })
            .for_each(|f| match f {
                AstNode::Assignment((ident, _), _) => variables.push(ident.clone()),
                AstNode::FunctionDeclaration(name, args, return_type, _) => {
                    functions_docs.insert(
                        name.to_string(),
                        format!(
                            "fn {name}({}): {return_type}",
                            args.iter()
                                .map(|(name, r#type)| format!("{name}: {type}"))
                                .collect::<Vec<String>>()
                                .join(",")
                        ),
                    );
                    functions.push(name.to_string());
                }
                _ => {}
            });

        let id = param.text_document_position.position.line as usize;
        let character = param.text_document_position.position.character as usize;
        let line = match self.file.get(&id) {
            Some(file) => file.to_string(),
            None => String::new(),
        };

        let mut text = String::new();

        for (i, char) in line.split("").enumerate() {
            let c = char.chars().next();
            if c.is_some() && !c.unwrap().is_ascii_alphanumeric() {
                text = String::new();
                continue;
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
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: functions_docs.get(f).unwrap().to_string(),
                    })),
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

        self.get_closest_match(
            &text,
            STD.iter().map(|f| f.to_string()).collect::<Vec<String>>(),
        )
        .iter()
        .for_each(|f| {
            let type_map = internal_type_map(f);

            responses.push(CompletionItem {
                label: f.to_string(),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "fn {f}({}): {}",
                        type_map
                            .0
                            .iter()
                            .map(|g| g
                                .iter()
                                .map(|h| h.to_string())
                                .collect::<Vec<String>>()
                                .join("|"))
                            .collect::<Vec<String>>()
                            .join(","),
                        type_map.1
                    ),
                })),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            })
        });

        Ok(Some(CompletionResponse::Array(responses)))
    }

    async fn hover(&self, param: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hover requested!")
            .await;

        let file = &self
            .file
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let mut variables = vec![
            ("pi".to_string(), PI.to_string()),
            ("π".to_string(), PI.to_string()),
            ("e".to_string(), E.to_string()),
        ];

        let mut functions = vec![];
        let functions_docs = DashMap::new();

        let std_docs = STD.map(|f| f.to_string());

        Parser::new(Lexer::new(file).tokens())
            .ast()
            .unwrap_or_default()
            .iter()
            .filter(|f| {
                matches!(
                    f,
                    AstNode::Assignment(..) | AstNode::FunctionDeclaration(..)
                )
            })
            .for_each(|f| {
                if let AstNode::Assignment((ident, datatype), _) = f {
                    variables.push((
                        ident.clone(),
                        datatype.unwrap_or(NumberType::Unknown).to_string(),
                    ))
                }
                if let AstNode::FunctionDeclaration(name, args, return_type, _) = f {
                    functions_docs.insert(
                        name.to_string(),
                        format!(
                            "fn {name}({}): {return_type}",
                            args.iter()
                                .map(|(name, r#type)| format!("{name}: {type}"))
                                .collect::<Vec<String>>()
                                .join(",")
                        ),
                    );
                    functions.push(name.to_string());
                }
            });

        let line = param.text_document_position_params.position.line as usize;
        let character = param.text_document_position_params.position.character as usize;
        let line = self.file.get(&line).unwrap().to_string();
        let mut text = String::new();
        let mut pos_found = false;

        for (i, char) in line.split("").enumerate() {
            let c = char.chars().next();
            if c.is_some() && !c.unwrap().is_ascii_alphanumeric() {
                if pos_found {
                    break;
                }
                text = String::new();
                continue;
            }
            text += char;
            if i == character {
                pos_found = true;
            }
        }

        let response = if functions.binary_search(&text).is_ok() {
            MarkedString::String(functions_docs.get(&text).unwrap().to_string())
        } else if std_docs.contains(&text.trim().to_string()) {
            let type_map = internal_type_map(&text);

            MarkedString::String(format!(
                "fn {text}({}): {}",
                type_map
                    .0
                    .iter()
                    .map(|g| g
                        .iter()
                        .map(|h| h.to_string())
                        .collect::<Vec<String>>()
                        .join("|"))
                    .collect::<Vec<String>>()
                    .join(","),
                type_map.1
            ))
        } else if variables.iter().any(|f| f.0 == text) {
            let (name, r#type) = variables.iter().find(|f| f.0 == text).unwrap();
            MarkedString::String(format!("{name}: {type}"))
        } else {
            MarkedString::String(String::new())
        };

        Ok(Some(Hover {
            contents: HoverContents::Array(vec![response]),
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
