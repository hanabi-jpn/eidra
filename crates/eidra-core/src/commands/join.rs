use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use eidra_transport::crypto::{derive_shared_secret, generate_keypair};

pub async fn run(room_id: &str, port: u16) -> anyhow::Result<()> {
    println!("Connecting to room {}...", room_id);

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    println!("Connected | Room: {} | E2EE: X25519+ChaCha20", room_id);

    // Key exchange: receive their public key first, then send ours
    let mut their_public_bytes = [0u8; 32];
    stream.read_exact(&mut their_public_bytes).await?;

    let keypair = generate_keypair();
    let our_public = keypair.public_key.as_bytes().to_owned();
    stream.write_all(&our_public).await?;

    let their_public = x25519_dalek::PublicKey::from(their_public_bytes);
    let shared_secret = derive_shared_secret(keypair.secret_key(), &their_public);

    super::escape::chat_loop_external(stream, shared_secret).await
}
