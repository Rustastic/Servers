use std::collections::HashMap;
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::server::{Server, ServerType};


pub enum CommunicationType{
    GetRegisteredClients,
    Register,
    SendMessage
}

pub struct CommunicationMessage{
    message_type: CommunicationType,
    sender_id: NodeId,
    receiver_id: NodeId,
    message: String
}

struct CommunicationServer {
    id: NodeId,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    media_server_type: ServerType,
    registered_clients: HashMap<NodeId, Vec<NodeId>> //note id of the sender and the path to the receiver
}

impl Server for CommunicationServer{
    fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>) -> Self {
        Self {
            id,
            packet_recv,
            packet_send,
            media_server_type: ServerType::Communication,
            registered_clients: HashMap::new()
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

impl CommunicationServer{
    fn register_client(&mut self, client_id: NodeId, path: Vec<NodeId>){
        self.registered_clients.insert(client_id, path);
    }

    fn send_registered_clients(&self) -> Vec<NodeId>{
        self.registered_clients.keys().clone().collect()
    }

    fn send_message(&self, message: CommunicationMessage){
        //send the message to the receiver
        let path = self.registered_clients.get(&message.receiver_id).unwrap();
        //Fragment the packet todo!()
        let mut packet = Packet::new();
        packet.set_source(self.id);
        packet.set_destination(message.receiver_id);
        packet.set_payload(message.message.as_bytes().to_vec());
        for node in path{
            packet.add_hop(*node);
        }
        self.packet_send.get(&path[0]).unwrap().send(packet);
    }


}