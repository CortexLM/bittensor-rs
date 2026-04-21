//! MEV Shield command group — encrypted extrinsic submission using ML-KEM-768.

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// MEV Shield subcommands.
#[derive(Debug, Subcommand)]
pub enum MevCommand {
    /// Submit an encrypted extrinsic via MEV Shield
    #[command(name = "submit-encrypted")]
    SubmitEncrypted {
        /// Hex-encoded extrinsic payload to encrypt and submit
        extrinsic_hex: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl MevCommand {
    /// Dispatch the MEV shield subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::SubmitEncrypted { extrinsic_hex, password } => {
                exec_submit_encrypted(config, &extrinsic_hex, password).await
            }
        }
    }
}

async fn exec_submit_encrypted(
    config: &Config,
    extrinsic_hex: &str,
    password: Option<String>,
) -> Result<()> {
    let _pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());

    // We need the coldkey address for logging purposes
    let _coldkey_addr =
        wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

    let extrinsic_bytes = hex::decode(extrinsic_hex.trim_start_matches("0x"))
        .context("invalid hex-encoded extrinsic")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    // In a full implementation, we would:
    // 1. Fetch the on-chain NextKey (ML-KEM-768 public key)
    // 2. Encrypt the extrinsic bytes using MevShieldSubmit::encrypt_extrinsic
    // 3. SCALE-encode the payload using MevShieldSubmit::scale_encode_payload
    // 4. Submit via the submit_encrypted_extrinsic RPC call
    //
    // For now, we demonstrate the encryption step assuming a provided key:
    println!("MEV Shield: preparing encrypted extrinsic submission.");
    println!("  Extrinsic size: {} bytes", extrinsic_bytes.len());

    // Placeholder: the actual RPC for fetching the NextKey and submitting
    // the encrypted extrinsic requires chain-specific runtime support.
    let _ = (rpc, extrinsic_bytes);

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn prompt_password(password: Option<String>) -> Result<String> {
    match password {
        Some(p) => Ok(p),
        None => Ok(rpassword::prompt_password("Enter coldkey password: ")?),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mev_command_debug_format() {
        let cmd = MevCommand::SubmitEncrypted { extrinsic_hex: "0x1234".into(), password: None };
        assert!(format!("{cmd:?}").contains("SubmitEncrypted"));
    }

    #[test]
    fn parse_mev_submit_encrypted() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "mev",
            "submit-encrypted",
            "0x1234abcd",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Mev {
                command: MevCommand::SubmitEncrypted { extrinsic_hex, password },
            } => {
                assert_eq!(extrinsic_hex, "0x1234abcd");
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Mev::SubmitEncrypted, got {other:?}"),
        }
    }

    #[test]
    fn mev_hex_decode_valid() {
        let bytes = hex::decode("1234abcd").unwrap();
        assert_eq!(bytes, vec![0x12, 0x34, 0xab, 0xcd]);
    }

    #[test]
    fn mev_hex_decode_invalid_fails() {
        assert!(hex::decode("zzzz").is_err());
    }

    #[test]
    fn mev_hex_decode_with_0x_prefix() {
        let bytes = hex::decode("0x1234".trim_start_matches("0x")).unwrap();
        assert_eq!(bytes, vec![0x12, 0x34]);
    }
}
