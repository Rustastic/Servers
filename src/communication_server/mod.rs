mod packet_cache;
mod handle_command_packet;
mod send_functions;

use std::collections::HashMap;
use std::thread;
use assembler::HighLevelMessageFactory;
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::network::{NodeId};
use wg_2024::packet::{ NodeType, Packet};

use messages;
use messages::high_level_messages::{ ServerType};
use messages::high_level_messages::ServerType::Chat;
use messages::server_commands::{CommunicationServerCommand, CommunicationServerEvent};
use source_routing::Router;
use crate::communication_server::packet_cache::PacketCache;
use crate::server::Server;

pub struct CommunicationServer {
    pub id: NodeId,
    pub router: Router,
    pub message_factory: HighLevelMessageFactory,
    pub packet_cache: PacketCache, //cache for the packets
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub controller_send: Sender<CommunicationServerEvent>,
    controller_recv: Receiver<CommunicationServerCommand>,
    server_type: ServerType,
    registered_clients: Vec<NodeId>, //note id of the sender and the path to the receiver
}

impl Server for CommunicationServer {
     fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>, controller_send: Sender<CommunicationServerEvent>, controller_recv: Receiver<CommunicationServerCommand>) -> Self {
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
    fn run(&mut self) {
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

    fn reinit_network(&mut self) {
        self.router.clear_routing_table();
        self.flood_network();
    }
    fn flood_network(&self){
        for sender in self.packet_send.values(){
            let req = self.router.get_flood_request();
            self.send_packet(req, Some(sender));
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }
}


