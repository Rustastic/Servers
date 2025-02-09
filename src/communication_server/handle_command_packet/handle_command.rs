use colored::Colorize;
use log::{error, info, warn};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use messages::{
    server_commands::{CommunicationServerCommand, CommunicationServerEvent},
    high_level_messages::ServerMessage,
};
use crate::communication_server::CommunicationServer;
use crate::server::Server;

impl CommunicationServer {
    /// Handles commands directed at the communication server.
    pub fn handle_command(&mut self, command: CommunicationServerCommand) {
        match command {
            CommunicationServerCommand::InitFlooding => self.flood_network(),
            CommunicationServerCommand::StartServer => {
                info!("{} [ CommunicationServer {} ]: Server started successfully.", "✔".green(), self.id);
            }
            CommunicationServerCommand::StopServer => {
                info!("{} [ CommunicationServer {} ]: Server stopped successfully.", "✔".green(), self.id);
            }
            CommunicationServerCommand::RemoveSender(id) => {
                let _ = self
                    .packet_send
                    .remove(&id);
            }
            CommunicationServerCommand::AddSender(id, sender) => {
                if let std::collections::hash_map::Entry::Vacant(e) = self.packet_send.entry(id) {
                    e.insert(sender);
                    info!("{} [ CommunicationServer {} ]: Sender added successfully.", "✔".green(), self.id);
                } else {
                    warn!(
                        "{} [ CommunicationServer {} ] is already connected to [ Drone {id} ]",
                        "!!!".yellow(),
                        self.id
                    );
                }
            }
        }
    }
}
