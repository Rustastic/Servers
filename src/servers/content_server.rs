use std::collections::HashMap;
use std::thread;
use assembler::HighLevelMessageFactory;
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet};

use messages;
use messages::high_level_messages::ServerType;
use messages::high_level_messages::ServerType::{Text};
use messages::server_commands::{CommunicationServerCommand, CommunicationServerEvent};
use source_routing::Router;
use crate::servers::packet_cache::PacketCache;

pub struct ContentServer {
    pub id: NodeId,
    pub router: Router,
    pub message_factory: HighLevelMessageFactory,
    pub packet_cache: PacketCache,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub controller_send: Sender<CommunicationServerEvent>,
    pub controller_recv: Receiver<CommunicationServerCommand>,
    pub server_type: ServerType, //text or media
    pub file_list: HashMap<String, String>, //file name and file path
}

impl ContentServer {
    pub fn new(id: NodeId,
           packet_recv: Receiver<Packet>,
           packet_send: HashMap<NodeId, Sender<Packet>>,
           controller_send: Sender<CommunicationServerEvent>,
           controller_recv: Receiver<CommunicationServerCommand>,
           server_type: ServerType) -> Self {
        let mut hm = HashMap::new();
        match server_type {
            //inizialize the hashmap
            Text => {
                let prefix ="text_files/".to_string();
                hm.insert("file1".to_string(),prefix.clone()+"file1.txt");
                hm.insert("file2".to_string(),prefix.clone()+"file2.txt");
                hm.insert("file3".to_string(),prefix.clone()+"file3.txt");
                hm.insert("file4".to_string(),prefix.clone()+"file4.txt");
                hm.insert("file5".to_string(),prefix.clone()+"file5.txt");
            }
            ServerType::Media => {
                let prefix ="media_files/".to_string();
                hm.insert("media1".to_string(),prefix.clone()+"media1.jpg");
                hm.insert("media2".to_string(),prefix.clone()+"media2.jpg");
                hm.insert("media3".to_string(),prefix.clone()+"media3.jpg");
                hm.insert("media4".to_string(),prefix.clone()+"media4.jpg");
                hm.insert("media5".to_string(),prefix.clone()+"media5.jpg");
            }
            _ => {}
        }
        Self {
            id,
            router: Router::new(id, NodeType::Server),
            packet_cache: PacketCache::new(),
            message_factory: HighLevelMessageFactory::new(id, NodeType::Server),
            packet_recv,
            packet_send,
            controller_send,
            controller_recv,
            server_type,
            file_list: hm,
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

    pub fn reinit_network(&mut self) {
        self.router.clear_routing_table();
        self.flood_network();
    }
    pub fn flood_network(&self) {
        for sender in self.packet_send.values() {
            let req = self.router.get_flood_request();
            self.send_packet(req, Some(sender));
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }
}


