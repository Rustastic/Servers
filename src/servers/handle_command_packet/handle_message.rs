use colored::Colorize;
use log::{error, info};
use messages::high_level_messages::{ClientMessage, Message, ServerMessage};
use messages::high_level_messages::MessageContent::{FromClient, FromServer};
use messages::high_level_messages::ServerMessage::{ServerType};
use wg_2024::network::{NodeId};
use crate::servers::communication_server::CommunicationServer;
use crate::servers::content_server::ContentServer;

impl CommunicationServer {
    pub fn handle_message(&mut self, message: Message) {
        info!("{}, CommunicationServer {}, Recived a packet {:?}", "✔".green(), self.id, message);
        let FromClient(content) = message.content else {
            error!("{} [ CommunicationServer {} ]: Received message is not from a client.", "✗".red(), self.id);
            return;
        };

        match content {
            ClientMessage::GetServerType => {
                // Retrieve and send server type to the client
                let server_type = self.server_type;
                let server_message = ServerType(server_type);
                self.send_message_to_client(&server_message, message.source_id);
            }
            ClientMessage::RegisterToChat => {
                // Handle client registration to chat
                if self.registered_clients.contains(&message.source_id) {
                    error!("{} [ CommunicationServer {} ]: Client {} already registered to chat", "✗".red(), self.id, message.source_id);
                } else {
                    self.registered_clients.push(message.source_id);
                    info!("{}, CommunicationServer {}, Client {} registered to chat", "✔".green(), self.id, message.source_id);
                }
            }

            ClientMessage::Logout => {
                // Handle client logout
                if let Some(index) = self.registered_clients.iter().position(|&id| id == message.source_id) {
                    self.registered_clients.remove(index);
                    info!("{}, CommunicationServer {}, Client {} logged out", "✔".green(), self.id, message.source_id);
                } else {
                    error!("{} [ CommunicationServer {} ]: Client {} not registered to chat", "✗".red(), self.id, message.source_id);
                }
            }
            ClientMessage::GetClientList => {
                // Retrieve and send the list of clients to the requester
                let client_list = self.registered_clients.clone();
                self.send_message_to_client(&ServerMessage::ClientList(client_list), message.source_id);
            }
            ClientMessage::SendMessage { recipient_id, content } => {
                // Send message to the recipient
                if self.registered_clients.contains(&recipient_id) && self.registered_clients.contains(&message.source_id) {
                    let server_message = ServerMessage::MessageReceived {
                        sender_id: message.source_id,
                        content,
                    };
                    self.send_message_to_client(&server_message, recipient_id);
                } else {
                    error!("{} [ CommunicationServer {} ]: Client {} is not registered to chat", "✗".red(), self.id, recipient_id);
                }
            }

            ClientMessage::GetFilesList | ClientMessage::GetFile(_) | ClientMessage::GetMedia(_) => {
                error!("{} [ CommunicationServer {} ]: This is not a MediaServer, wrong request", "✗".red(), self.id);
            }
        }
    }

    fn send_message_to_client(&mut self, server_message: &ServerMessage, destination_id: NodeId) {
        let Ok(header) = self.router.get_source_routing_header(destination_id) else {
            error!(
                "{} [ Communication {} ]: Cannot send message, destination {} is unreachable",
                "✗".red(),
                self.id,
                   destination_id );
            return;
        };
        for fragment_packet in self.message_factory.get_message_from_message_content(
            FromServer(server_message.clone()),
            &header,
            destination_id,
        ) {
            self.packet_cache.insert_packet(&fragment_packet);
            self.send_packet(fragment_packet, None);
        }
        info!("Message sent to client {}: {:?}", destination_id, server_message);
    }
}

impl ContentServer {
    pub fn handle_message(&mut self, message: Message) {
        let FromClient(content) = message.content else {
            error!("{} [ CommunicationServer {} ]: Received message is not from a client.", "✗".red(), self.id);
            return;
        };

        match content {
            ClientMessage::GetServerType => {
                // Retrieve and send server type to the client
                let server_type = self.server_type;
                let server_message = ServerType(server_type);
                self.send_message_to_client(&server_message, message.source_id);
            }
            ClientMessage::GetFilesList => {
                let files_list = self.file_list.keys().cloned().collect();
                self.send_message_to_client(&ServerMessage::FilesList(files_list), message.source_id);
            }

            ClientMessage::GetFile(file_name) | ClientMessage::GetMedia(file_name) => {
                if let Some(file_path) = self.file_list.get(&file_name) {
                    match std::fs::read_to_string(file_path) {
                        Ok(file_content) => {
                            let file_size = file_content.len();
                            let server_message = ServerMessage::File {
                                file_id: file_name.clone(),
                                size: file_size,
                                content: file_content,
                            };
                            self.send_message_to_client(&server_message, message.source_id);
                        }
                        Err(e) => {
                            error!(
                    "{} [ ContentServer {} ]: Failed to read file {}: {}",
                    "✗".red(),
                    self.id,
                    file_name,
                    e
                );
                        }
                    }
                } else {
                    error!(
            "{} [ ContentServer {} ]: File {} not found",
            "✗".red(),
            self.id,
            file_name
        );
                }
            }

            ClientMessage::RegisterToChat | ClientMessage::Logout | ClientMessage::GetClientList => {
                error!("{} [ CommunicationServer {} ]: This is not a ChatServer, wrong request", "✗".red(), self.id);
            }
            ClientMessage::SendMessage { recipient_id: _recipient_id, content: _content } => {
                error!("{} [ CommunicationServer {} ]: This is not a ChatServer, wrong request", "✗".red(), self.id);
            }
        }
    }

    fn send_message_to_client(&mut self, server_message: &ServerMessage, destination_id: NodeId) {
        let Ok(header) = self.router.get_source_routing_header(destination_id) else {
            error!(
                "{} [ Communication {} ]: Cannot send message, destination {} is unreachable",
                "✗".red(),
                self.id,
                   destination_id );
            return;
        };
        for fragment_packet in self.message_factory.get_message_from_message_content(
            FromServer(server_message.clone()),
            &header,
            destination_id,
        ) {
            self.packet_cache.insert_packet(&fragment_packet);
            self.send_packet(fragment_packet, None);
        }
        info!("Message sent to client {}: {:?}", destination_id, server_message);
    }
}
