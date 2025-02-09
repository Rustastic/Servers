use colored::Colorize;
use crossbeam_channel::Sender;
use log::{error, info};
use messages::server_commands::CommunicationServerEvent;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::communication_server::CommunicationServer;

impl CommunicationServer {
    pub fn send_packet(&self, msg: Packet, sender: Option<&Sender<Packet>>) {
        match msg.pack_type {
            wg_2024::packet::PacketType::Ack(_)
            | wg_2024::packet::PacketType::Nack(_)
            | wg_2024::packet::PacketType::FloodResponse(_) => self.send_or_shortcut(msg),
            wg_2024::packet::PacketType::FloodRequest(_) => {
                let Some(sender) = sender else { return };
                self.send_to_sender(msg, sender);
            }
            wg_2024::packet::PacketType::MsgFragment(_) => {
                let Some(dest) = msg.routing_header.next_hop() else {
                    error!(
                        "{} [MediaClienet {}] error taking next_hop",
                        "✗".red(),
                        self.id
                    );
                    return;
                };
                self.send_to_neighbour_id(msg, dest);
            }
        }
    }

    pub fn send_to_sender(&self, msg: Packet, sender: &Sender<Packet>) {
        info!("{} [CommunicationServer {}] sending packet", "✓".green(), self.id);
        sender
            .send(msg)
            .inspect_err(|e| {
                self.send_controller(CommunicationServerEvent::SendError(e.clone()));
                error!(
                    "{} [CommunicationServer {}] error in sending packet (session: {}, fragment: {})",
                    "✗".red(),
                    self.id,
                    e.0.session_id,
                    e.0.get_fragment_index()
                );
            })
            .ok();
    }

    fn send_to_neighbour_id(&self, msg: Packet, neighbour_id: NodeId) {
        let Some(sender) = self.packet_send.get(&neighbour_id) else {
            error!(
                "{} [ CommunicationServer {} ]: Cannot send message, destination {neighbour_id} is unreachable",
                "✗".red(),
                self.id,
            );
            return;
        };
        self.send_to_sender(msg, sender);
    }

    fn send_or_shortcut(&self, msg: Packet) {
        match self.get_sender(&msg) {
            Some(sender) => {
                sender
                    .send(msg)
                    .inspect_err(|e| {
                        self.send_controller(CommunicationServerEvent::ControllerShortcut(e.0.clone()));
                    })
                    .ok();
            }
            None => self.send_controller(CommunicationServerEvent::ControllerShortcut(msg)),
        }
    }
    fn get_sender(&self, packet: &Packet) -> Option<Sender<Packet>> {
        Some(
            self.packet_send
                .get(&packet.routing_header.next_hop()?)?
                .clone(),
        )
    }

    pub fn send_controller(&self, msg: CommunicationServerEvent) {
        self.controller_send
            .send(msg)
            .inspect_err(|e| {
                error!(
                    "{} [CommunicationServer {}] error in sending to sim-controller. Message: [{:?}]",
                    "✗".red(),
                    self.id,
                    e.0
                );
            })
            .ok();
    }
}
