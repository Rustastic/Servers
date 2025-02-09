use std::collections::HashMap;
use crossbeam_channel::{Receiver, Sender};
use messages::server_commands::{CommunicationServerCommand, CommunicationServerEvent};
use wg_2024::{
    network::{NodeId},
    packet::{Packet},
};


pub trait Server{
    fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>, controller_send: Sender<CommunicationServerEvent>, controller_recv: Receiver<CommunicationServerCommand>) -> Self ;
    fn run(&mut self);
    fn reinit_network(&mut self);
    fn flood_network(&self);
}
