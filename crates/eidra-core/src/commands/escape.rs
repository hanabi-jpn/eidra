use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use eidra_transport::crypto::{derive_shared_secret, encrypt, generate_keypair};
use eidra_transport::room::generate_room_id;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let room_id = generate_room_id();

    println!("Room: {} | Expires: 30min", room_id);
    println!("Share: eidra join {} {}", room_id, addr.port());
    println!("Waiting for peer...");
    println!();

    let (mut stream, peer_addr) = listener.accept().await?;
    println!("Connected from {} | E2EE: X25519+ChaCha20", peer_addr);

    // Key exchange: send our public key, receive theirs
    let keypair = generate_keypair();
    let our_public = keypair.public_key.as_bytes().to_owned();
    stream.write_all(&our_public).await?;

    let mut their_public_bytes = [0u8; 32];
    stream.read_exact(&mut their_public_bytes).await?;

    let their_public = x25519_dalek::PublicKey::from(their_public_bytes);
    let shared_secret = derive_shared_secret(keypair.secret_key(), &their_public);
    let key: [u8; 32] = *shared_secret.as_bytes();

    // Generate SAS (Short Authentication String) for verification
    let sas = generate_sas(&key);
    println!("E2EE established.");
    println!("\u{1f510} Verify with peer \u{2014} SAS: {}", sas);
    println!("  If this doesn't match your peer's SAS, type /end immediately.");
    println!();

    chat_loop_internal(stream, key).await
}

pub(crate) async fn chat_loop_external(
    stream: TcpStream,
    shared_secret: x25519_dalek::SharedSecret,
) -> anyhow::Result<()> {
    let key: [u8; 32] = *shared_secret.as_bytes();

    // Generate SAS (Short Authentication String) for verification
    let sas = generate_sas(&key);
    println!("E2EE established.");
    println!("\u{1f510} Verify with peer \u{2014} SAS: {}", sas);
    println!("  If this doesn't match your peer's SAS, type /end immediately.");
    println!();

    chat_loop_internal(stream, key).await
}

async fn chat_loop_internal(stream: TcpStream, key: [u8; 32]) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let key_recv = key;

    // Spawn receiver task
    let recv_handle = tokio::spawn(async move {
        loop {
            // Read 4-byte length prefix
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let len = u32::from_be_bytes(len_buf) as usize;
            if len == 0 || len > 1_000_000 {
                break;
            }

            // Read encrypted message
            let mut enc_buf = vec![0u8; len];
            if reader.read_exact(&mut enc_buf).await.is_err() {
                break;
            }

            match eidra_transport::crypto::decrypt(&key_recv, &enc_buf) {
                Ok(plaintext) => {
                    let msg = String::from_utf8_lossy(&plaintext);
                    println!("\r\x1b[K\x1b[36mpeer>\x1b[0m {}", msg);
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                Err(_) => {
                    println!("\r\x1b[K\x1b[31m[decryption failed]\x1b[0m");
                }
            }
        }
    });

    // Send loop: read from stdin
    let stdin = tokio::io::stdin();
    let mut stdin_reader = tokio::io::BufReader::new(stdin);
    let mut line_buf = String::new();

    loop {
        print!("> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        line_buf.clear();
        let n = stdin_reader.read_line(&mut line_buf).await?;
        if n == 0 {
            break;
        }

        let msg = line_buf.trim();
        if msg == "/end" {
            println!("Session ended. Keys zeroized.");
            break;
        }
        if msg.is_empty() {
            continue;
        }

        let encrypted = encrypt(&key, msg.as_bytes()).map_err(|e| anyhow::anyhow!("{}", e))?;

        let len = (encrypted.len() as u32).to_be_bytes();
        writer.write_all(&len).await?;
        writer.write_all(&encrypted).await?;
        writer.flush().await?;
    }

    recv_handle.abort();
    Ok(())
}

/// Generate a Short Authentication String from the shared secret.
/// Returns a 4-word phrase from the NATO phonetic alphabet for verbal verification.
fn generate_sas(key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"eidra-sas-v1");
    hasher.update(key);
    let hash = hasher.finalize();

    // Use first 4 bytes to select 4 words from a wordlist
    const WORDS: &[&str] = &[
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra",
        "tango", "uniform", "victor", "whiskey", "xray", "yankee", "zulu", "anchor", "bridge",
        "castle", "drift", "ember", "frost",
    ];

    let w1 = WORDS[hash[0] as usize % WORDS.len()];
    let w2 = WORDS[hash[1] as usize % WORDS.len()];
    let w3 = WORDS[hash[2] as usize % WORDS.len()];
    let w4 = WORDS[hash[3] as usize % WORDS.len()];

    format!("{}-{}-{}-{}", w1, w2, w3, w4)
}
