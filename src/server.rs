use std::collections::HashMap;
use std::println;
use crossbeam_channel::{Receiver, Sender};
use messages::high_level_messages::ServerMessage;
use messages::server_commands::CommunicationServerEvent;
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType},
};


pub trait Server{
    fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>, controller_send: Sender<CommunicationServerEvent>, controller_recv: Receiver<ServerMessage>) -> Self ;
    fn run(&mut self);
    fn reinit_network(&mut self);
    fn flood_network(&self);
}
