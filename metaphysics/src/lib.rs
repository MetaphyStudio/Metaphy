use std::sync::Arc;

use godot::prelude::*;
use metaphy_network::{init_debug_interface, libp2p::futures::StreamExt};
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
        }
    }

    fn ready(&mut self) {
        init_debug_interface();

        self.runtime().spawn(async {
            let node = metaphy_network::Phylosopher::new(None).expect("Failed to create P2P Node!");
            node.bind(None).await;

            let swarm = node.get_swarm();

            loop {
                select! {
                    swarm_event = async {
                        let mut swarm = swarm.lock().await;
                        swarm.select_next_some().await
                    } => {
                        match swarm_event {
                            metaphy_network::libp2p::swarm::SwarmEvent::Behaviour(behaviour) => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::NewListenAddr { listener_id, address } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ExpiredListenAddr { listener_id, address } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ListenerClosed { listener_id, addresses, reason } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::Dialing { peer_id, connection_id } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
                            metaphy_network::libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => todo!(),
                            _ => todo!(),
                        }
                    }
                }
            }
        });
    }
}

#[godot_api]
impl Metaphy {
    pub fn runtime(&self) -> Arc<Runtime> {
        Arc::clone(&self.runtime)
    }
}
