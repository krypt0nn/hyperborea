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

pub async fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;

    std::io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let mut scrollbar_state = ScrollbarState::new(0)
        .position(0);

    let mut message_widget = TextArea::new(vec![]);

    message_widget.set_placeholder_text("Type a message...");

    loop {
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

            let mut items = Vec::new();

            for i in 1..101 {
                items.push(Line::from(i.to_string()));
            }

            let paragraph = Paragraph::new(items.clone())
                .scroll((0, 0))
                .block(Block::new().borders(Borders::RIGHT));

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar_state = scrollbar_state.content_length(items.len());

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

                    else if key.code == KeyCode::Enter {
                        // todo: send
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

    Ok(())
}