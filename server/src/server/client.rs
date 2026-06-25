use crate::command::{ActiveCommand, Command, CommandQueue};
use crate::constants::MAX_PENDING_COMMANDS;
use std::io::{self, Write};
use std::net::TcpStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    WaitingTeamName,
    Ai,
    Gui,
}

pub struct Client {
    pub socket: TcpStream,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
    pub output_offset: usize,
    pub state: ClientState,
    pub team_name: Option<String>,
    pub player_id: Option<usize>,
    pub queue: CommandQueue,
    pub active: Option<ActiveCommand>,
    pub close_after_flush: bool,
}

impl Client {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            input: Vec::new(),
            output: Vec::new(),
            output_offset: 0,
            state: ClientState::WaitingTeamName,
            team_name: None,
            player_id: None,
            queue: CommandQueue::default(),
            active: None,
            close_after_flush: false,
        }
    }

    pub fn queue_text(&mut self, text: &str) {
        self.output.extend_from_slice(text.as_bytes());
    }

    pub fn has_pending_output(&self) -> bool {
        self.output_offset < self.output.len()
    }

    pub fn outstanding_commands(&self) -> usize {
        self.queue.len() + usize::from(self.active.is_some())
    }

    pub fn enqueue_command(&mut self, command: Command) -> bool {
        if self.outstanding_commands() >= MAX_PENDING_COMMANDS {
            return false;
        }
        self.queue.push_back(command);
        true
    }

    pub fn flush_output(&mut self) -> io::Result<()> {
        while self.has_pending_output() {
            match self.socket.write(&self.output[self.output_offset..]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "socket stopped accepting output",
                    ));
                }
                Ok(written) => self.output_offset += written,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        if self.output_offset == self.output.len() {
            self.output.clear();
            self.output_offset = 0;
        }
        Ok(())
    }

    pub fn clear_commands(&mut self) {
        self.queue.clear();
        self.active = None;
    }
}
