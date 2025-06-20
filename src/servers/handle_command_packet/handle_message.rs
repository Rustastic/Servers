use crate::servers::communication_server::CommunicationServer;
use crate::servers::content_server::ContentServer;
use base64::{engine::general_purpose, Engine as _};
use colored::Colorize;
use image::ImageReader;
use log::{error, info};
use messages::high_level_messages::MessageContent::{FromClient, FromServer};
use messages::high_level_messages::ServerMessage::ServerType;
use messages::high_level_messages::{ClientMessage, Message, ServerMessage};
use std::io::Cursor;
use wg_2024::network::NodeId;

impl CommunicationServer {
    #[allow(clippy::too_many_lines)]
    pub fn handle_message(&mut self, message: Message) {
        info!(
            "{}, CommunicationServer {}, Recived a packet {:?}",
            "✔".green(),
            self.id,
            message
        );
        let FromClient(content) = message.content else {
            error!(
                "{} [ CommunicationServer {} ]: Received message is not from a client.",
                "✗".red(),
                self.id
            );
            return;
        };

        match content {
            ClientMessage::GetServerType => {
                // Retrieve and send server type to the client
                let server_message = ServerType(self.server_type);
                self.send_message_to_client(&server_message, message.source_id);
            }
            ClientMessage::RegisterToChat => {
                // Handle client registration to chat
                if self.registered_clients.contains(&message.source_id) {
                    error!(
                        "{} [ CommunicationServer {} ]: Client {} already registered to chat",
                        "✗".red(),
                        self.id,
                        message.source_id
                    );
                } else {
                    self.registered_clients.push(message.source_id);
                    self.send_message_to_client(
                        &ServerMessage::SuccessfulRegistration,
                        message.source_id,
                    );
                    info!(
                        "{}, CommunicationServer {}, Client {} registered to chat",
                        "✔".green(),
                        self.id,
                        message.source_id
                    );
                }
            }

            ClientMessage::Logout => {
                // Handle client logout
                if let Some(index) = self
                    .registered_clients
                    .iter()
                    .position(|&id| id == message.source_id)
                {
                    self.registered_clients.remove(index);
                    self.send_message_to_client(
                        &ServerMessage::SuccessfullLogOut,
                        message.source_id,
                    );
                    info!(
                        "{}, CommunicationServer {}, Client {} logged out",
                        "✔".green(),
                        self.id,
                        message.source_id
                    );
                } else {
                    error!(
                        "{} [ CommunicationServer {} ]: Client {} not registered to chat",
                        "✗".red(),
                        self.id,
                        message.source_id
                    );
                }
            }
            ClientMessage::GetClientList => {
                // Retrieve and send the list of clients to the requester
                let client_list = self.registered_clients.clone();
                self.send_message_to_client(
                    &ServerMessage::ClientList(client_list),
                    message.source_id,
                );
            }
            ClientMessage::SendMessage {
                recipient_id,
                content,
            } => {
                // Send message to the recipient
                if self.registered_clients.contains(&recipient_id)
                    && self.registered_clients.contains(&message.source_id)
                {
                    let server_message = ServerMessage::MessageReceived {
                        sender_id: message.source_id,
                        content,
                    };
                    self.send_message_to_client(&server_message, recipient_id);
                } else {
                    self.send_message_to_client(
                        &ServerMessage::UnreachableClient(message.source_id),
                        recipient_id,
                    );
                    error!(
                        "{} [ CommunicationServer {} ]: Client {} is not registered to chat",
                        "✗".red(),
                        self.id,
                        recipient_id
                    );
                }
            }

            ClientMessage::GetFilesList
            | ClientMessage::GetFile(_)
            | ClientMessage::GetMedia(_) => {
                error!(
                    "{} [ CommunicationServer {} ]: This is not a MediaServer, wrong request",
                    "✗".red(),
                    self.id
                );
            }
        }
    }

    fn send_message_to_client(&mut self, server_message: &ServerMessage, destination_id: NodeId) {
        let Ok(header) = self.router.get_source_routing_header(destination_id) else {
            error!(
                "{} [ Communication {} ]: Cannot send message, destination {} is unreachable",
                "✗".red(),
                self.id,
                destination_id
            );
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
        info!("Message sent to client {destination_id}: {server_message:?}");
    }
}

impl ContentServer {
    #[allow(clippy::too_many_lines)]
    pub fn handle_message(&mut self, message: Message) {
        let FromClient(content) = message.content else {
            error!(
                "{} [ CommunicationServer {} ]: Received message is not from a client.",
                "✗".red(),
                self.id
            );
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
                self.send_message_to_client(
                    &ServerMessage::FilesList(files_list),
                    message.source_id,
                );
            }
            ClientMessage::GetMedia(file_name) => {
                // println!("[MediaServer {}] received GetMedia({file_name})", self.id);
                let file_path_t = self.file_list.get(&file_name).unwrap_or(&file_name);
                let Ok(dir) = std::env::current_dir()
                    .inspect_err(|e| self.print_error(&file_name, &e.to_string()))
                else {
                    return;
                };
                let file_path = dir.join("src").join("data_files").join(file_path_t);
                let Ok(file_content) = ImageReader::open(file_path)
                    .inspect_err(|e| self.print_error(&file_name, &e.to_string()))
                else {
                    return;
                };
                let Ok(file_media_content) = file_content.decode().inspect_err(|e| {
                    self.print_error(&file_name, &e.to_string());
                }) else {
                    return;
                };
                let mut buf = Vec::new();
                if file_media_content
                    .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
                    .inspect_err(|e| self.print_error(&file_name, &e.to_string()))
                    .is_ok()
                {
                    let base_64 = general_purpose::STANDARD.encode(&buf);
                    let server_message = ServerMessage::Media(file_name.clone(), base_64);
                    self.send_message_to_client(&server_message, message.source_id);
                }
            }
            ClientMessage::GetFile(file_name) => {
                if let Some(file_path_t) = self.file_list.get(&file_name) {
                    match std::env::current_dir() {
                        Ok(dir) => {
                            let file_path = dir.join("src").join("text_files").join(file_path_t);
                            info!("reading file: {:?}", dir.display());
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
                                    self.print_error(&file_name, &e.to_string());
                                }
                            }
                        }
                        Err(_e) => {}
                    }
                } else {
                    // self.print_error(&file_name);
                }
            }
            ClientMessage::RegisterToChat
            | ClientMessage::Logout
            | ClientMessage::GetClientList => {
                error!(
                    "{} [ CommunicationServer {} ]: This is not a ChatServer, wrong request",
                    "✗".red(),
                    self.id
                );
            }
            ClientMessage::SendMessage {
                recipient_id: _recipient_id,
                content: _content,
            } => {
                error!(
                    "{} [ CommunicationServer {} ]: This is not a ChatServer, wrong request",
                    "✗".red(),
                    self.id
                );
            }
        }
    }

    fn print_error(&self, file_name: &str, e: &String) {
        // println!(
        //     "{} [ ContentServer {} ]: Failed to read file {}, error: {e}",
        //     "✗".red(),
        //     self.id,
        //     file_name
        // );
        error!(
            "{} [ ContentServer {} ]: Failed to read file {}, error: {e}",
            "✗".red(),
            self.id,
            file_name
        );
    }

    fn send_message_to_client(&mut self, server_message: &ServerMessage, destination_id: NodeId) {
        let Ok(header) = self.router.get_source_routing_header(destination_id) else {
            error!(
                "{} [ Content {} ]: Cannot send message, destination {} is unreachable",
                "✗".red(),
                self.id,
                destination_id
            );
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
        info!("Message sent to client {destination_id}: {server_message:?}");
    }
}
