use assembler::HighLevelMessageFactory;
use crossbeam_channel::{select_biased, Receiver, Sender};
use messages;
use messages::high_level_messages::ServerType;
use messages::high_level_messages::ServerType::Chat;
use messages::server_commands::{CommunicationServerCommand, CommunicationServerEvent};
use packet_cache::PacketCache;
use source_routing::Router;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet};

pub struct CommunicationServer {
    pub id: NodeId,
    pub router: Router,
    pub message_factory: HighLevelMessageFactory,
    pub packet_cache: PacketCache, //cache for the packets
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub controller_send: Sender<CommunicationServerEvent>,
    pub controller_recv: Receiver<CommunicationServerCommand>,
    pub server_type: ServerType,
    pub registered_clients: Vec<NodeId>, //note id of the sender and the path to the receiver
}

impl CommunicationServer {
    #[must_use]
    pub fn new(
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        controller_send: Sender<CommunicationServerEvent>,
        controller_recv: Receiver<CommunicationServerCommand>,
    ) -> Self {
        Self {
            id,
            router: Router::new(id, NodeType::Server),
            packet_cache: PacketCache::new(),
            message_factory: HighLevelMessageFactory::new(id, NodeType::Server),
            packet_recv,
            packet_send,
            controller_send,
            controller_recv,
            server_type: Chat,
            registered_clients: vec![],
        }
    }
    pub fn run(&mut self) {
        self.flood_network();
        loop {
            select_biased! {
                recv(self.packet_recv) -> packet => {
                    if let Ok(packet) = packet {
                        self.handle_packet(packet);
                    }
                },
                recv(self.controller_recv) -> command => {
                    if let Ok(command) = command {
                        self.handle_command(command);
                    }
                }
            }
        }
    }

    pub fn flood_network(&mut self) {
        let requests = self.router.get_flood_requests(self.packet_send.len());
        for (sender, request) in self.packet_send.values().zip(requests) {
            self.send_packet(request, Some(sender));
        }
        thread::sleep(Duration::from_secs(2));
    }
}
