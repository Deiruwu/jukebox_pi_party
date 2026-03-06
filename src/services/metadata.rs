use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use crate::model::{ApiResponse, Request, Track};


pub struct MetadataClient {
    addr: String,
}

impl MetadataClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            addr: format!("{}:{}", host, port),
        }
    }

    /// Función para enviar y recibir (el "corazón" que pedías)
    pub async fn call(&self, action: &str, query: &str) -> Result<Vec<Track>, Box<dyn Error>> {
        // 1. Conectar
        let mut stream = TcpStream::connect(&self.addr).await?;
        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);

        // 2. Preparar y enviar JSON (con newline al final)
        let req = Request {
            action: action.to_string(),
            query: query.to_string(),
        };
        let mut payload = serde_json::to_vec(&req)?;
        payload.push(b'\n');

        writer.write_all(&payload).await?;
        writer.flush().await?;

        // 3. Recibir respuesta
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        // 3. Primero deserializamos la respuesta global
        let api_res: ApiResponse = serde_json::from_str(&line)?;

        if api_res.status == "ok" {
            Ok(api_res.data.unwrap_or_default())
        } else {
            let err_msg = api_res.message.unwrap_or_else(|| "Unknown error".to_string());
            Err(err_msg.into())
        }
    }
}
