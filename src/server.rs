use std::collections::HashMap;
use crossbeam_channel::{Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType},
};


pub trait Server{
    fn new(id: NodeId, packet_recv: Receiver<Packet>, packet_send: HashMap<NodeId, Sender<Packet>>) -> Self;
    fn run(&mut self);
    fn handle_packet(&mut self, packet: Packet);
}
