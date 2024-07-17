const MAX_CHAT_MESSAGES_COUNT: usize = 128;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use ratatui::prelude::*;
use ratatui::widgets::*;

use ratatui::crossterm::ExecutableCommand;

use ratatui::crossterm::terminal::{
    EnterAlternateScreen,
    LeaveAlternateScreen,
    enable_raw_mode,
    disable_raw_mode
};

use ratatui::crossterm::event::{
    Event,
    KeyEventKind,
    KeyCode,
    KeyModifiers,
    poll as poll_event,
    read as read_event
};

use tui_textarea::TextArea;

use hyperelm::prelude::*;

use crate::params::Params;
use crate::*;

pub async fn run(client: Arc<ChatMemberApp>, endpoint: ClientEndpoint, params: Params) -> anyhow::Result<()> {
    // Logic

    let mut last_history_id = 0;

    let chat_messages_widgets = Arc::new(Mutex::new(Vec::new()));
    let chat_members_names = Arc::new(Mutex::new(HashMap::new()));

    let response = client.request(endpoint.clone(), ChatMemberRequest::Join { username: params.room_username }).await
        .map_err(|err| anyhow::anyhow!("Failed to join chat room: {err}"))?;

    if let ChatHosterResponse::JoinResponse { members, history } = response {
        // todo
    }

    tokio::spawn({
        let client = client.clone();
        let endpoint = endpoint.clone();

        let chat_messages_widgets = chat_messages_widgets.clone();

        async move {
            loop {
                // TODO: handle errors
                let response = client.request(endpoint.clone(), ChatMemberRequest::GetHistory {
                    since_id: last_history_id + 1
                }).await;

                if let Ok(ChatHosterResponse::History { history }) = response {
                    let mut chat_messages_widgets = chat_messages_widgets.lock().await;

                    for record in history {
                        match &record.body {
                            ChatHistoryBlockBody::MemberJoin { public_key, username } => {
                                let widget = Line::from(format!("[{username}] joined chat"))
                                    .centered()
                                    .yellow();

                                chat_members_names.lock().await
                                    .insert(public_key.clone(), username.clone());

                                chat_messages_widgets.push(widget);
                            }

                            ChatHistoryBlockBody::MemberLeave { public_key } => {
                                let name = chat_members_names.lock().await
                                    .get(public_key)
                                    .cloned()
                                    .unwrap_or_else(move || public_key.to_base64());

                                let widget = Line::from(format!("[{name}] left chat"))
                                    .centered()
                                    .yellow();

                                chat_messages_widgets.push(widget);
                            }

                            ChatHistoryBlockBody::MemberSendMessage { public_key, message } => {
                                let name = chat_members_names.lock().await
                                    .get(public_key)
                                    .cloned()
                                    .unwrap_or_else(move || public_key.to_base64());

                                let widget = Line::from(format!("[{name}] : {message}"));

                                chat_messages_widgets.push(widget);
                            }
                        }

                        last_history_id = record.id;
                    }

                    while chat_messages_widgets.len() > MAX_CHAT_MESSAGES_COUNT {
                        chat_messages_widgets.remove(0);
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(params.room_sync_delay)).await;
            }
        }
    });

    // TUI

    enable_raw_mode()?;

    std::io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let mut scrollbar_state = ScrollbarState::new(0)
        .position(0);

    let mut message_widget = TextArea::new(vec![]);

    message_widget.set_placeholder_text("Type a message...");

    // Main loop

    loop {
        let chat_messages_widgets = chat_messages_widgets.lock().await.clone();
        let messages_content_length = chat_messages_widgets.len();

        terminal.draw(|frame| {
            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3)
            ]).split(frame.size());

            let top_layout = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(16)
            ]).split(layout[0]);

            let messages_block = Block::bordered()
                .padding(Padding::symmetric(2, 1))
                .title("Messages");

            let members_block = Block::bordered().title("Members");
            let message_block = Block::bordered().title("Your message");

            let messages_area = messages_block.inner(top_layout[0]);
            let members_area = members_block.inner(top_layout[1]);
            let message_area = message_block.inner(layout[1]);

            frame.render_widget(messages_block, top_layout[0]);
            frame.render_widget(members_block, top_layout[1]);
            frame.render_widget(message_block, layout[1]);

            frame.render_widget(message_widget.widget(), message_area);

            let paragraph = Paragraph::new(chat_messages_widgets)
                .scroll((0, 0))
                .block(Block::new().borders(Borders::RIGHT));

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar_state = scrollbar_state.content_length(messages_content_length);

            frame.render_widget(paragraph, messages_area);

            frame.render_stateful_widget(
                scrollbar,
                messages_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state
            );
        })?;

        if poll_event(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = read_event()? {
                if key.kind == KeyEventKind::Press {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if key.code == KeyCode::Char('c') {
                            break;
                        }
                    }

                    else if key.code == KeyCode::Up {
                        scrollbar_state.prev();
                    }

                    else if key.code == KeyCode::Down {
                        scrollbar_state.next();
                    }

                    // Send chat message
                    else if key.code == KeyCode::Enter {
                        message_widget.select_all();
                        message_widget.cut();

                        let message = message_widget.yank_text()
                            .trim()
                            .to_string();

                        if !message.is_empty() {
                            // TODO: some fancy UI handling?
                            let _ = client.send(endpoint.clone(), ChatMemberMessage::SendMessage {
                                message: message_widget.yank_text()
                            }).await;
                        }
                    }

                    else {
                        message_widget.input(key);
                    }
                }
            }
        }
    }

    disable_raw_mode()?;

    std::io::stdout().execute(LeaveAlternateScreen)?;

    // Send leave message if possible
    let _ = client.send(endpoint, ChatMemberMessage::Leave).await;

    Ok(())
}