use crate::servers::communication_server::CommunicationServer;
use crate::servers::content_server::ContentServer;
use base64::{engine::general_purpose, Engine};
use colored::Colorize;
use image::ImageReader;
use log::{error, info};
use messages::high_level_messages::MessageContent::{FromClient, FromServer};
use messages::high_level_messages::ServerMessage::ServerType;
use messages::high_level_messages::{ClientMessage, Message, ServerMessage};
use std::io::Cursor;
use std::path::PathBuf;
use wg_2024::network::NodeId;

pub fn read_file(file_path: PathBuf) -> Result<String, String> {
    println!("{}", file_path.display());
    match ImageReader::open(file_path.clone()) {
        Ok(file_content) => match file_content.decode() {
            Ok(file_media_content) => {
                let mut buf = Vec::new();
                // Salva l'immagine in memoria come JPEG
                match file_media_content.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg) {
                    Ok(..) => {
                        // Codifica in Base64
                        let base_64 = general_purpose::STANDARD.encode(&buf);
                        // Esempio: usiamo semplicemente file_path come "nome"
                        let server_message = ServerMessage::Media(file_path.iter().last().unwrap().to_str().unwrap().to_string(), base_64);
                        println!("Sending file: {:?}", server_message);
                        Ok("ok".to_string())
                    },
                    Err(e) => Err(format!("Error in encoding image: {}", e))
                }
            },
            Err(e) => Err(format!("Error in decoding image: {}", e)),
        },
        Err(e) => Err(format!("Error in opening image: {}", e)),
    }
}

// Sezione test
#[cfg(test)]
mod tests {
    // Porta dentro tutto il contenuto pubblico del modulo principale
    use super::*;

    #[test]
    fn test_read_file() {
        let dir = std::env::current_dir().unwrap();
        assert_eq!(read_file(dir.join("/text_files/file1.html")), Ok("ok".to_string()));    
    }
}
