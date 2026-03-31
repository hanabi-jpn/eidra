use std::path::PathBuf;

use tracing::info;

/// Returns the Eidra home directory (~/.eidra/).
fn eidra_home() -> anyhow::Result<PathBuf> {
    let home = dirs_or_home()?;
    Ok(home.join(".eidra"))
}

fn dirs_or_home() -> anyhow::Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))
}

/// Initialize Eidra: generate a local CA certificate, create config directory,
/// copy default config and policy files, and detect Ollama.
pub async fn run() -> anyhow::Result<()> {
    let eidra_dir = eidra_home()?;

    // 1. Create ~/.eidra/ directory
    if eidra_dir.exists() {
        info!(path = %eidra_dir.display(), "Eidra directory already exists");
    } else {
        std::fs::create_dir_all(&eidra_dir)?;
        info!(path = %eidra_dir.display(), "Created Eidra directory");
    }

    // 2. Generate CA certificate with rcgen
    let ca_cert_path = eidra_dir.join("ca.pem");
    let ca_key_path = eidra_dir.join("ca-key.pem");

    if ca_cert_path.exists() && ca_key_path.exists() {
        info!("CA certificate already exists, skipping generation");
    } else {
        info!("Generating local CA certificate for HTTPS interception...");
        let (cert_pem, key_pem) = generate_ca_cert()?;
        std::fs::write(&ca_cert_path, &cert_pem)?;
        std::fs::write(&ca_key_path, &key_pem)?;

        // Set restrictive permissions on the private key
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ca_key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        info!("CA certificate written to {}", ca_cert_path.display());
        info!("CA private key written to {}", ca_key_path.display());
    }

    // 3. Copy default config.yaml
    let config_path = eidra_dir.join("config.yaml");
    if config_path.exists() {
        info!("config.yaml already exists, skipping");
    } else {
        let default_config = include_str!("../../../../config/default.yaml");
        std::fs::write(&config_path, default_config)?;
        info!("Default config written to {}", config_path.display());
    }

    // 4. Copy default policy.yaml
    let policy_path = eidra_dir.join("policy.yaml");
    if policy_path.exists() {
        info!("policy.yaml already exists, skipping");
    } else {
        let default_policy = include_str!("../../../../config/policies/default.yaml");
        std::fs::write(&policy_path, default_policy)?;
        info!("Default policy written to {}", policy_path.display());
    }

    // 5. Check for Ollama
    let ollama_available = check_ollama().await;

    // 6. Print setup instructions
    println!();
    println!("  eidra init complete!");
    println!();
    println!("  Created: {}", eidra_dir.display());
    println!("    - ca.pem        (CA certificate for HTTPS interception)");
    println!("    - ca-key.pem    (CA private key)");
    println!("    - config.yaml   (proxy & scan configuration)");
    println!("    - policy.yaml   (data handling policies)");
    println!();

    if ollama_available {
        println!("  Ollama detected at localhost:11434");
        println!("  Local LLM routing is available. Enable it in config.yaml:");
        println!("    local_llm:");
        println!("      enabled: true");
    } else {
        println!("  Ollama not detected at localhost:11434");
        println!("  To enable local LLM routing, install Ollama:");
        println!("    https://ollama.com/download");
    }

    println!();
    println!("  Next steps:");
    println!("    1. Trust the CA certificate:");
    println!("       macOS:  sudo security add-trusted-cert -d -r trustRoot \\");
    println!(
        "                 -k /Library/Keychains/System.keychain {}",
        ca_cert_path.display()
    );
    println!("       Linux:  sudo cp {} /usr/local/share/ca-certificates/eidra-ca.crt && sudo update-ca-certificates", ca_cert_path.display());
    println!();
    println!("    2. Start the proxy:");
    println!("       eidra start");
    println!();
    println!("    3. Configure your tools to use the proxy:");
    println!("       export HTTPS_PROXY=http://127.0.0.1:8080");
    println!();

    Ok(())
}

/// Generate a self-signed CA certificate using rcgen.
fn generate_ca_cert() -> anyhow::Result<(String, String)> {
    use rcgen::{CertificateParams, DistinguishedName, KeyPair};

    let mut params = CertificateParams::default();

    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "Eidra Local CA");
    dn.push(rcgen::DnType::OrganizationName, "Eidra");
    params.distinguished_name = dn;

    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];

    // CA cert valid for 10 years
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(3650);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    Ok((cert_pem, key_pem))
}

/// Check if Ollama is available at localhost:11434.
async fn check_ollama() -> bool {
    matches!(
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::TcpStream::connect("127.0.0.1:11434"),
        )
        .await,
        Ok(Ok(_))
    )
}
