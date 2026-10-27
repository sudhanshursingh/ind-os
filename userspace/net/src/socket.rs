//! Capability-Gated Socket Management Table (SP-51, REQ-KRN-110, INV-NET-001..003)

use crate::types::{
    Ipv4Address, NetError, SocketType, TcpState,
    CAP_NET_BIND, CAP_NET_CONNECT, CAP_NET_LISTEN, CAP_NET_RECV, CAP_NET_SEND, CAP_NET_SOCKET,
    MAX_SOCKETS, SOCKET_BUF_SIZE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketCapToken {
    pub socket_id: u32,
    pub rights: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Socket {
    pub id: u32,
    pub sock_type: SocketType,
    pub state: TcpState,
    pub local_ip: Ipv4Address,
    pub local_port: u16,
    pub remote_ip: Ipv4Address,
    pub remote_port: u16,
    pub rx_buf: [u8; SOCKET_BUF_SIZE],
    pub rx_len: usize,
    pub tx_buf: [u8; SOCKET_BUF_SIZE],
    pub tx_len: usize,
}

impl Socket {
    pub const fn empty(id: u32) -> Self {
        Self {
            id,
            sock_type: SocketType::Stream,
            state: TcpState::Closed,
            local_ip: Ipv4Address::any(),
            local_port: 0,
            remote_ip: Ipv4Address::any(),
            remote_port: 0,
            rx_buf: [0u8; SOCKET_BUF_SIZE],
            rx_len: 0,
            tx_buf: [0u8; SOCKET_BUF_SIZE],
            tx_len: 0,
        }
    }
}

pub struct SocketTable {
    sockets: [Option<Socket>; MAX_SOCKETS],
    next_id: u32,
}

impl SocketTable {
    pub const fn new() -> Self {
        Self {
            sockets: [None; MAX_SOCKETS],
            next_id: 1,
        }
    }

    /// Create socket requiring CAP_NET_SOCKET capability (INV-NET-001)
    pub fn create_socket(
        &mut self,
        sock_type: SocketType,
        cap_rights: u32,
    ) -> Result<SocketCapToken, NetError> {
        if (cap_rights & CAP_NET_SOCKET) == 0 {
            return Err(NetError::PermissionDenied);
        }

        for slot in self.sockets.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;

                let mut s = Socket::empty(id);
                s.sock_type = sock_type;
                *slot = Some(s);

                return Ok(SocketCapToken {
                    socket_id: id,
                    rights: cap_rights,
                });
            }
        }
        Err(NetError::QueueFull)
    }

    /// Bind socket to local port requiring CAP_NET_BIND (INV-NET-001, INV-NET-003)
    pub fn bind(
        &mut self,
        token: &SocketCapToken,
        ip: Ipv4Address,
        port: u16,
    ) -> Result<(), NetError> {
        if (token.rights & CAP_NET_BIND) == 0 {
            return Err(NetError::PermissionDenied);
        }

        // Check if port in use
        for slot in self.sockets.iter() {
            if let Some(s) = slot {
                if s.local_port == port && s.id != token.socket_id {
                    return Err(NetError::PortInUse);
                }
            }
        }

        let sock = self.get_mut(token.socket_id)?;
        sock.local_ip = ip;
        sock.local_port = port;
        Ok(())
    }

    /// Transition to listen state requiring CAP_NET_LISTEN (INV-NET-001)
    pub fn listen(&mut self, token: &SocketCapToken) -> Result<(), NetError> {
        if (token.rights & CAP_NET_LISTEN) == 0 {
            return Err(NetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        if sock.sock_type != SocketType::Stream {
            return Err(NetError::StateMismatch);
        }
        sock.state = TcpState::Listen;
        Ok(())
    }

    /// Connect to remote host requiring CAP_NET_CONNECT (INV-NET-001)
    pub fn connect(
        &mut self,
        token: &SocketCapToken,
        remote_ip: Ipv4Address,
        remote_port: u16,
    ) -> Result<(), NetError> {
        if (token.rights & CAP_NET_CONNECT) == 0 {
            return Err(NetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        sock.remote_ip = remote_ip;
        sock.remote_port = remote_port;

        if sock.sock_type == SocketType::Stream {
            sock.state = TcpState::SynSent;
        }
        Ok(())
    }

    /// Send data requiring CAP_NET_SEND (INV-NET-001)
    pub fn send(&mut self, token: &SocketCapToken, data: &[u8]) -> Result<usize, NetError> {
        if (token.rights & CAP_NET_SEND) == 0 {
            return Err(NetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        if sock.sock_type == SocketType::Stream && sock.state != TcpState::Established {
            return Err(NetError::StateMismatch);
        }

        let to_copy = if data.len() > SOCKET_BUF_SIZE {
            SOCKET_BUF_SIZE
        } else {
            data.len()
        };

        sock.tx_buf[..to_copy].copy_from_slice(&data[..to_copy]);
        sock.tx_len = to_copy;
        Ok(to_copy)
    }

    /// Receive data requiring CAP_NET_RECV (INV-NET-001)
    pub fn recv(&mut self, token: &SocketCapToken, out_buf: &mut [u8]) -> Result<usize, NetError> {
        if (token.rights & CAP_NET_RECV) == 0 {
            return Err(NetError::PermissionDenied);
        }

        let sock = self.get_mut(token.socket_id)?;
        let len = sock.rx_len;
        if len == 0 {
            return Ok(0);
        }

        let to_read = if len > out_buf.len() {
            out_buf.len()
        } else {
            len
        };

        out_buf[..to_read].copy_from_slice(&sock.rx_buf[..to_read]);
        sock.rx_len = 0;
        Ok(to_read)
    }

    pub fn get(&self, socket_id: u32) -> Result<&Socket, NetError> {
        for slot in self.sockets.iter() {
            if let Some(s) = slot {
                if s.id == socket_id {
                    return Ok(s);
                }
            }
        }
        Err(NetError::SocketNotFound)
    }

    pub fn get_mut(&mut self, socket_id: u32) -> Result<&mut Socket, NetError> {
        for slot in self.sockets.iter_mut() {
            if let Some(s) = slot {
                if s.id == socket_id {
                    return Ok(s);
                }
            }
        }
        Err(NetError::SocketNotFound)
    }

    pub fn close(&mut self, socket_id: u32) -> Result<(), NetError> {
        for slot in self.sockets.iter_mut() {
            if let Some(s) = slot {
                if s.id == socket_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(NetError::SocketNotFound)
    }
}
