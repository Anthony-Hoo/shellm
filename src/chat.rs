use std::io::{self, Write};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::llm::{ChatMessage, ChatReply, LLMClient, Role};

fn prompt(buf: &str) {
    print!("\r\x1b[2Kyou> {buf}");
    io::stdout().flush().ok();
}

pub fn chat_mode(llm: &dyn LLMClient) -> Result<Option<String>> {
    print!(
        "\r\n\x1b[2K[LLM chat] Type your question. Ctrl+L accepts the command. Ctrl+C exits.\r\n"
    );

    let mut history: Vec<ChatMessage> = Vec::new();
    let mut last_cmd: Option<String> = None;
    let mut buf = String::new();

    prompt(&buf);

    loop {
        let evt = event::read()?;
        if let Event::Key(key) = evt {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter => {
                    print!("\r\n");
                    io::stdout().flush().ok();

                    let line = buf.trim_end().to_string();
                    if line.is_empty() {
                        buf.clear();
                        prompt(&buf);
                        continue;
                    }

                    let response: ChatReply = llm.chat(&history, &line)?;
                    let cmd = response.suggested_command.clone().unwrap_or("".to_string());
                    print!("assistant> {}\r\n", response.text.trim());
                    print!("\x1b[2Kcandidate: {cmd}\r\n");

                    if !cmd.is_empty() {
                        last_cmd = Some(cmd);
                    }
                    history.push(ChatMessage {
                        role: Role::User,
                        content: line,
                    });
                    history.push(ChatMessage {
                        role: Role::Assistant,
                        content: response.text,
                    });

                    buf.clear();
                    prompt(&buf);
                }
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(cmd) = last_cmd {
                        return Ok(Some(cmd));
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Backspace => {
                    if !buf.is_empty() {
                        buf.pop();
                        prompt(&buf);
                    }
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    prompt(&buf);
                }
                _ => {}
            }
        }
    }
}
