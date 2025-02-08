use std::collections::HashMap;
use assembler::HighLevelMessageFactory;
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Fragment, NodeType, Packet};

use crate::server::{Server};
use messages;
use messages::high_level_messages::{ServerMessage, ServerType};
use messages::high_level_messages::ServerType::Chat;
use messages::server_commands::CommunicationServerEvent;
use source_routing::Router;
use crate::packet_cache::PacketCache;

pub struct CommunicationServer {
    pub id: NodeId,
    router: Router,
    message_factory: HighLevelMessageFactory,
    packet_cache: PacketCache, //cache for the packets
    packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub controller_send: Sender<CommunicationServerEvent>,
    controller_recv: Receiver<ServerMessage>,
    server_type: ServerType,
    registered_clients: Vec<NodeId>, //note id of the sender and the path to the receiver
}

impl Server for CommunicationServer {
     fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>, controller_send: Sender<CommunicationServerEvent>, controller_recv: Receiver<ServerMessage>) -> Self {
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
                recv(self.controller_recv) -> message => {
                    if let Ok(message) = message {
                        self.handle_message(message);
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
    }
}

impl CommunicationServer{
    fn handle_packet(&mut self, packet: Packet) {
        //todo!()
        //need to assemble the package
        //one it's full assamble the message I'm gonna see what to do with it
    }

    fn handle_message(&mut self, message: ServerMessage){
        //todo!()
    }

}

