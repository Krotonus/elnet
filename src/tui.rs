use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub cursor_position: usize,
}

pub struct ChatMessage {
    pub sender: String,
    pub content: String,
}

impl ChatState {
    pub fn new() -> Self {
        ChatState {
            messages: Vec::new(),
            input: String::new(),
            cursor_position: 0,
        }
    }

    pub fn add_message(&mut self, sender: String, content: String) {
        self.messages.push(ChatMessage { sender, content });
    }
}

pub fn draw(frame: &mut Frame, state: &ChatState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Chat history
    let messages: Vec<ListItem> = state
        .messages
        .iter()
        .map(|msg| {
            let content = format!("{}: {}", msg.sender, msg.content);
            ListItem::new(Line::from(vec![
                Span::styled(
                    content,
                    Style::default().fg(if msg.sender == "You" {
                        Color::Green
                    } else {
                        Color::Blue
                    }),
                )
            ]))
        })
        .collect();

    let messages = List::new(messages)
        .block(Block::default().title("Chat History").borders(Borders::ALL));
    frame.render_widget(messages, chunks[0]);

    // Input box
    let input = Paragraph::new(state.input.as_str())
        .style(Style::default())
        .block(Block::default().title("Input").borders(Borders::ALL));
    frame.render_widget(input, chunks[1]);
} 