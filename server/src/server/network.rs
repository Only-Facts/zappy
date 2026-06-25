use crate::constants::{READ_BUFFER_SIZE, WELCOME_MESSAGE};
use crate::game::GameState;
use crate::server::client::{Client, ClientId};
use std::io::{self, Read};
use std::net::TcpListener;
use std::os::fd::AsRawFd;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollTarget {
    Listener,
    Client(ClientId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollEvent {
    pub target: PollTarget,
    pub readable: bool,
    pub writable: bool,
    pub closed: bool,
}

pub fn poll_events(
    listener: &TcpListener,
    game: &GameState,
    timeout: Option<Duration>,
) -> io::Result<Vec<PollEvent>> {
    let mut entries = Vec::with_capacity(game.clients.len() + 1);
    entries.push((
        PollTarget::Listener,
        libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
    ));

    for (client_id, client) in &game.clients {
        let mut events = libc::POLLIN;
        if client.has_pending_output() {
            events |= libc::POLLOUT;
        }
        entries.push((
            PollTarget::Client(*client_id),
            libc::pollfd {
                fd: client.socket.as_raw_fd(),
                events,
                revents: 0,
            },
        ));
    }

    let mut poll_fds = entries
        .iter()
        .map(|(_, poll_fd)| *poll_fd)
        .collect::<Vec<_>>();
    let timeout_ms = timeout_to_poll_milliseconds(timeout);
    let result = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            timeout_ms,
        )
    };

    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    if result == 0 {
        return Ok(Vec::new());
    }

    let closed_mask = libc::POLLERR | libc::POLLHUP | libc::POLLNVAL;
    let events = entries
        .into_iter()
        .zip(poll_fds)
        .filter_map(|((target, _), poll_fd)| {
            (poll_fd.revents != 0).then_some(PollEvent {
                target,
                readable: poll_fd.revents & libc::POLLIN != 0,
                writable: poll_fd.revents & libc::POLLOUT != 0,
                closed: poll_fd.revents & closed_mask != 0,
            })
        })
        .collect();
    Ok(events)
}

fn timeout_to_poll_milliseconds(timeout: Option<Duration>) -> libc::c_int {
    let Some(timeout) = timeout else {
        return -1;
    };
    if timeout.is_zero() {
        return 0;
    }

    let fractional_milliseconds = timeout.subsec_nanos().div_ceil(1_000_000);
    let milliseconds = timeout
        .as_secs()
        .saturating_mul(1_000)
        .saturating_add(u64::from(fractional_milliseconds));
    milliseconds.min(libc::c_int::MAX as u64) as libc::c_int
}

pub fn accept_connections(listener: &TcpListener, game: &mut GameState) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((socket, _address)) => {
                socket.set_nonblocking(true)?;
                let client_id = game.allocate_client_id();
                let mut client = Client::new(socket);
                client.queue_text(WELCOME_MESSAGE);
                game.clients.insert(client_id, client);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

pub fn read_client(game: &mut GameState, client_id: ClientId) -> io::Result<bool> {
    let Some(client) = game.clients.get_mut(&client_id) else {
        return Ok(false);
    };
    let mut buffer = [0_u8; READ_BUFFER_SIZE];
    loop {
        match client.socket.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(read) => client.input.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

pub fn flush_client(game: &mut GameState, client_id: ClientId) -> io::Result<()> {
    if let Some(client) = game.clients.get_mut(&client_id) {
        client.flush_output()?;
    }
    Ok(())
}

pub fn flush_all_clients(game: &mut GameState) -> Vec<ClientId> {
    let client_ids = game.clients.keys().copied().collect::<Vec<_>>();
    client_ids
        .into_iter()
        .filter(|client_id| flush_client(game, *client_id).is_err())
        .collect()
}

pub fn remove_client(game: &mut GameState, client_id: ClientId) {
    game.handle_client_disconnect(client_id);
    game.clients.remove(&client_id);
}

#[cfg(test)]
mod tests {
    use super::timeout_to_poll_milliseconds;
    use std::time::Duration;

    #[test]
    fn poll_timeout_rounds_up_sub_milliseconds() {
        assert_eq!(
            timeout_to_poll_milliseconds(Some(Duration::from_micros(1))),
            1
        );
        assert_eq!(
            timeout_to_poll_milliseconds(Some(Duration::from_millis(2))),
            2
        );
        assert_eq!(timeout_to_poll_milliseconds(None), -1);
    }
}
