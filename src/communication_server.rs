use std::collections::HashMap;
use crossbeam_channel::{select_biased, Receiver, Sender};
use messages::{Message, ServerMessage, ServerType};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Fragment, Packet};

use crate::server::{Server};


struct CommunicationServer {
    id: NodeId,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    media_server_type: ServerType,
    registered_clients: HashMap<NodeId, Vec<NodeId>>, //note id of the sender and the path to the receiver
}

impl Server for CommunicationServer {
    fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>) -> Self {
        Self {
            id,
            packet_recv,
            packet_send,
            media_server_type: ServerType::ChatServer,
            registered_clients: HashMap::new(),
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
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        //todo!()
        //need to assemble the package
        //one it's full assamble the message I'm gonna see what to do with it
    }
}

impl CommunicationServer {
    fn register_client(&mut self, client_id: NodeId, path: Vec<NodeId>) {
        self.registered_clients.insert(client_id, path);
    }

    fn send_registered_clients(&self, destination_id: NodeId) -> Result<Ok(String), Box<dyn std::error::Error>> {
        self.send_message(Message::new_server_message(0, self.id, destination_id, ServerMessage::ClientList(self.registered_clients.clone().keys().collect())))
    }

    fn send_message(&self, message: Message) -> Result<Ok(String), Box<dyn std::error::Error>> {
        //send the message to the receiver
        if let Some(path) = self.registered_clients.get(&message.destination_id) {
            if !path.is_empty() {  // Check if path is not empty to avoid panic on path[0]
                if let Some(fragments) = message.serialize() {
                    let num_fragments = fragments.len();
                    for (fragment_index, fragment) in fragments.into_iter().enumerate() {
                        let packet = Packet::new_fragment(
                            SourceRoutingHeader::new(path, 1),
                            1,
                            Fragment::new(fragment_index, num_fragments, fragment),
                        );

                        if let Some(sender) = self.packet_send.get(&path[0]) {
                            if let Err(e) = sender.send(packet) {
                                println!("Error sending the packet: {:?} - Message: {:?}", e, message);
                            }else{
                                Ok("Message sent successfully".to_string())
                            }
                        } else {
                            println!("Error: No sender found for node {:?}", path[0]);
                        }
                    }
                } else {
                    println!("Error fragmenting the message: {:?}", message);
                }
            } else {
                println!("Error: Path for client {:?} is empty!", message.destination_id);
            }
        } else {
            println!("Error: Destination not found in registered clients ({:?})", message.destination_id);
        }
    }
}