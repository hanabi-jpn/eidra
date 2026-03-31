use std::io::Read;

use crate::runtime::{build_scanner, load_app_config, runtime_paths};

pub async fn run(path: Option<String>, json: bool) -> anyhow::Result<()> {
    let paths = runtime_paths()?;
    let config = load_app_config(&paths)?;
    let scanner = build_scanner(&config, true)?;

    let input = match path {
        Some(ref p) => std::fs::read_to_string(p)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let findings = scanner.scan(&input);

    if json {
        let payload = serde_json::json!({
            "findings_count": findings.len(),
            "findings": findings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if findings.is_empty() {
        println!("✓ No findings.");
    } else {
        for f in &findings {
            println!(
                "[{}] {} ({}) at offset {}",
                f.severity, f.rule_name, f.category, f.offset
            );
        }
        println!("\n{} finding(s) total.", findings.len());
    }

    Ok(())
}
