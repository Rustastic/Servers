use colored::Colorize;
use log::{info, warn};
use messages::{
    server_commands::{CommunicationServerCommand},
};
use messages::server_commands::ContentServerCommand;
use crate::servers::communication_server::CommunicationServer;
use crate::servers::content_server::{ContentServer};

impl CommunicationServer {
    /// Handles commands directed at the communication server.
    pub fn handle_command(&mut self, command: CommunicationServerCommand) {
        match command {
            CommunicationServerCommand::InitFlooding => self.flood_network(),
            CommunicationServerCommand::LogNetwork => self.router.log_network(),
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

impl ContentServer{
    pub fn handle_command(&mut self, command: ContentServerCommand) {
        match command {
            ContentServerCommand::InitFlooding => self.flood_network(),
            ContentServerCommand::RemoveSender(id) => {
                let _ = self
                    .packet_send
                    .remove(&id);
            }
            ContentServerCommand::AddSender(id, sender) => {
                if let std::collections::hash_map::Entry::Vacant(e) = self.packet_send.entry(id) {
                    e.insert(sender);
                    info!("{} [ ContentServer {} ]: Sender added successfully.", "✔".green(), self.id);
                } else {
                    warn!(
                        "{} [ ContentServer {} ] is already connected to [ Drone {id} ]",
                        "!!!".yellow(),
                        self.id
                    );
                }
            }
        }
    }
}
