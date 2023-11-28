use crate::message::Message;

#[derive(Default)]
pub struct Topic {
    pub name: String,
    pub messages: Vec<Message>,
    pub message_count: u64,
}

impl Topic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_msg(&mut self, message: Message) {
        self.messages.push(message);
        self.message_count += 1;
    }
}
