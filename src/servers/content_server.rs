use assembler::HighLevelMessageFactory;
use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet};

use messages;
use messages::high_level_messages::ServerType;
use messages::high_level_messages::ServerType::Text;
use messages::server_commands::{ContentServerCommand, ContentServerEvent};
use packet_cache::PacketCache;
use source_routing::Router;

pub struct ContentServer {
    pub id: NodeId,
    pub router: Router,
    pub message_factory: HighLevelMessageFactory,
    pub packet_cache: PacketCache,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub controller_send: Sender<ContentServerEvent>,
    pub controller_recv: Receiver<ContentServerCommand>,
    pub server_type: ServerType,            //text or media
    pub file_list: HashMap<String, String>, //file name and file path
}

impl ContentServer {
    #[must_use]
    pub fn new(
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        controller_send: Sender<ContentServerEvent>,
        controller_recv: Receiver<ContentServerCommand>,
        server_type: ServerType,
    ) -> Self {
        let mut hm = HashMap::new();
        match server_type {
            //inizialize the hashmap
            Text => {
                hm.insert("file1".to_string(), "file1.html".to_string());
                hm.insert("file2".to_string(), "file2.html".to_string());
                hm.insert("file3".to_string(), "file3.html".to_string());
                hm.insert("file4".to_string(), "file4.html".to_string());
                hm.insert("file5".to_string(), "file5.html".to_string());
            }
            ServerType::Media => {
                hm.insert("media1".to_string(), "media1.jpg".to_string());
                hm.insert("media2".to_string(), "media2.jpg".to_string());
                hm.insert("media3".to_string(), "media3.jpg".to_string());
                hm.insert("media4".to_string(), "media4.jpg".to_string());
                hm.insert("media5".to_string(), "media5.jpg".to_string());
            }
            ServerType::Chat => {}
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

    pub fn flood_network(&mut self) {
        let requests = self.router.get_flood_requests(self.packet_send.len());
        for (sender, request) in self.packet_send.values().zip(requests) {
            self.send_packet(request, Some(sender));
        }
        thread::sleep(Duration::from_secs(2));
    }
}
