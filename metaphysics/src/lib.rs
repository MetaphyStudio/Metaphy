use std::{num::NonZeroU32, sync::Arc, time::Duration};

use godot::prelude::*;
use metaphy_network::{
    init_debug_interface,
    libp2p::{
        core::{transport::ListenerId, ConnectedPoint},
        futures::StreamExt,
        swarm::{ConnectionError, ConnectionId, DialError, ListenError, SwarmEvent},
        Multiaddr, PeerId, TransportError,
    },
    Logic,
};
use tokio::{
    runtime::{self, Runtime},
    select,
};

struct Metaphysics;

#[gdextension]
unsafe impl ExtensionLibrary for Metaphysics {}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Metaphy {
    base: Base<Node>,
    runtime: Arc<Runtime>,
    swarm_event_channel: Option<tokio::sync::mpsc::Receiver<SwarmThreadEvent>>,
}

#[godot_api]
impl INode for Metaphy {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            runtime: Arc::new(
                runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio Runtime!"),
            ),
            swarm_event_channel: None,
        }
    }

    fn ready(&mut self) {
        init_debug_interface();

        // Create our channels for communication between the swarm/network thread and the main Godot thread.
        let (events_tx, events_rx) = tokio::sync::mpsc::channel::<SwarmThreadEvent>(32);

        // Creating a thread for our Node & Swarm.
        self.runtime().spawn(async move {
            let channel = events_tx;

            let node = metaphy_network::Phylosopher::new(None).expect("Failed to create P2P Node!");
            node.bind(None).await;

            let swarm = node.get_swarm();

            // let _ = channel.send(None);

            loop {
                select! {
                    swarm_event = async {
                        let mut swarm = swarm.lock().await;
                        swarm.select_next_some().await
                    } => {
                        match channel.send(swarm_event.into()).await {
                            Ok(_) => return,
                            Err(attempt) => println!("[ERROR] -> Failed to send {attempt:?} over the events channel to main thread!"),
                        }
                    }
                }
            }
        });

        // Add missing items to our main struct
        self.swarm_event_channel = Some(events_rx);
    }

    fn physics_process(&mut self, _deta: f64) {
        match self
            .swarm_event_channel
            .as_mut()
            .expect("There is no events channel to read from!")
            .try_recv()
        {
            Ok(event) => match event {
                SwarmThreadEvent::Behvaiour(event) => godot_print!("{:?}", event),
                // SwarmThreadEvent::ConnectionEstablished {
                //     peer_id,
                //     connection_id,
                //     endpoint,
                //     num_established,
                //     concurrent_dial_errors,
                //     established_in,
                // } => todo!(),
                // SwarmThreadEvent::ConnectionClosed {
                //     peer_id,
                //     connection_id,
                //     endpoint,
                //     num_established,
                //     cause,
                // } => todo!(),
                // SwarmThreadEvent::IncomingConnection {
                //     connection_id,
                //     local_addr,
                //     send_back_addr,
                // } => todo!(),
                // SwarmThreadEvent::IncomingConnectionError {
                //     connection_id,
                //     local_addr,
                //     send_back_addr,
                //     error,
                // } => todo!(),
                // SwarmThreadEvent::OutgoingConnectionError {
                //     connection_id,
                //     peer_id,
                //     error,
                // } => todo!(),
                SwarmThreadEvent::NewListenAddress {
                    listener_id,
                    address,
                } => godot_print!("New listen address: {}, {}", listener_id, address),
                // SwarmThreadEvent::ExpiredListenAddress {
                //     listener_id,
                //     address,
                // } => todo!(),
                // SwarmThreadEvent::ListenerClosed {
                //     listener_id,
                //     addresses,
                //     reason,
                // } => todo!(),
                // SwarmThreadEvent::ListenerError { listener_id, error } => todo!(),
                // SwarmThreadEvent::Dialing {
                //     peer_id,
                //     connection_id,
                // } => todo!(),
                // SwarmThreadEvent::NewExternalAddrCandidate { address } => todo!(),
                // SwarmThreadEvent::ExternalAddrConfirmed { address } => todo!(),
                // SwarmThreadEvent::ExternalAddrExpired { address } => todo!(),
                _ => (),
            },
            Err(error) => match error {
                tokio::sync::mpsc::error::TryRecvError::Empty => (),
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    panic!("[FETAL] -> The swarm thread event channel has been disconnected!\nThis could've happend do to the swarm thread crashing for some reason...")
                }
            },
        }
    }
}

#[godot_api]
impl Metaphy {
    pub fn runtime(&self) -> Arc<Runtime> {
        Arc::clone(&self.runtime)
    }
}

#[derive(Debug)]
enum SwarmThreadEvent {
    Behvaiour(Logic),
    ConnectionEstablished {
        peer_id: PeerId,
        connection_id: ConnectionId,
        endpoint: ConnectedPoint,
        num_established: NonZeroU32,
        concurrent_dial_errors: Option<Vec<(Multiaddr, TransportError<std::io::Error>)>>,
        established_in: Duration,
    },
    ConnectionClosed {
        peer_id: PeerId,
        connection_id: ConnectionId,
        endpoint: ConnectedPoint,
        num_established: u32,
        cause: Option<ConnectionError>,
    },
    IncomingConnection {
        connection_id: ConnectionId,
        local_addr: Multiaddr,
        send_back_addr: Multiaddr,
    },
    IncomingConnectionError {
        connection_id: ConnectionId,
        local_addr: Multiaddr,
        send_back_addr: Multiaddr,
        error: ListenError,
    },
    OutgoingConnectionError {
        connection_id: ConnectionId,
        peer_id: Option<PeerId>,
        error: DialError,
    },
    NewListenAddress {
        listener_id: ListenerId,
        address: Multiaddr,
    },
    ExpiredListenAddress {
        listener_id: ListenerId,
        address: Multiaddr,
    },
    ListenerClosed {
        listener_id: ListenerId,
        addresses: Vec<Multiaddr>,
        reason: Result<(), std::io::Error>,
    },
    ListenerError {
        listener_id: ListenerId,
        error: std::io::Error,
    },
    Dialing {
        peer_id: Option<PeerId>,
        connection_id: ConnectionId,
    },
    NewExternalAddrCandidate {
        address: Multiaddr,
    },
    ExternalAddrConfirmed {
        address: Multiaddr,
    },
    ExternalAddrExpired {
        address: Multiaddr,
    },
    None,
}

impl From<SwarmEvent<Logic>> for SwarmThreadEvent {
    fn from(value: SwarmEvent<Logic>) -> Self {
        match value {
            SwarmEvent::Behaviour(event) => Self::Behvaiour(event),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => Self::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            },
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => Self::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            },
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => Self::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            },
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => Self::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            },
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => Self::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            },
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => Self::NewListenAddress {
                listener_id,
                address,
            },
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => Self::ExpiredListenAddress {
                listener_id,
                address,
            },
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => Self::ListenerClosed {
                listener_id,
                addresses,
                reason,
            },
            SwarmEvent::ListenerError { listener_id, error } => {
                Self::ListenerError { listener_id, error }
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => Self::Dialing {
                peer_id,
                connection_id,
            },
            SwarmEvent::NewExternalAddrCandidate { address } => {
                Self::NewExternalAddrCandidate { address }
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                Self::ExternalAddrConfirmed { address }
            }
            SwarmEvent::ExternalAddrExpired { address } => Self::ExternalAddrExpired { address },
            _ => Self::None,
        }
    }
}
