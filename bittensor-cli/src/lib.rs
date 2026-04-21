//! bittensor-cli — CLI binary `btcli-rs` for Bittensor wallet and chain operations.

pub mod commands;
pub mod config;

use clap::Parser;

use commands::delegate::DelegateCommand;
use commands::metagraph::MetagraphCommand;
use commands::registration::{RegistrationCommand, RootCommand};
use commands::stake::StakeCommand;
use commands::subnet::SubnetCommand;
use commands::transfer::TransferCommand;
use commands::wallet::WalletCommand;
use commands::weights::WeightsCommand;

/// Top-level CLI structure.
#[derive(Debug, Parser)]
#[command(name = "btcli-rs")]
#[command(about = "Bittensor CLI (Rust)")]
#[command(version)]
pub struct Cli {
    /// Network to connect to: finney, test, local, archive, latent-lite
    #[arg(long, global = true)]
    pub network: Option<String>,

    /// Wallet name (defaults to "default")
    #[arg(long = "wallet.name", global = true)]
    pub wallet_name: Option<String>,

    /// Wallet base path (defaults to ~/.bittensor/wallets/)
    #[arg(long = "wallet.path", global = true)]
    pub wallet_path: Option<String>,

    /// Command to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level command enum.
#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Wallet management commands
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
    },

    /// Stake management commands
    Stake {
        #[command(subcommand)]
        command: StakeCommand,
    },

    /// Transfer commands
    Transfer {
        #[command(subcommand)]
        command: TransferCommand,
    },

    /// Registration commands (POW, burned, root)
    Register {
        #[command(subcommand)]
        command: RegistrationCommand,
    },

    /// Root network commands
    Root {
        #[command(subcommand)]
        command: RootCommand,
    },

    /// Subnet commands
    Subnet {
        #[command(subcommand)]
        command: SubnetCommand,
    },

    /// Delegate commands
    Delegate {
        #[command(subcommand)]
        command: DelegateCommand,
    },

    /// Weights commands (set-weights, get-weights)
    Weights {
        #[command(subcommand)]
        command: WeightsCommand,
    },

    /// Metagraph commands (show, sync)
    Metagraph {
        #[command(subcommand)]
        command: MetagraphCommand,
    },

    /// MEV Shield commands (encrypted extrinsic submission)
    #[cfg(feature = "mev")]
    Mev {
        #[command(subcommand)]
        command: commands::mev::MevCommand,
    },
}

impl Cli {
    /// Parse CLI args and run the selected command.
    pub async fn run(self) -> anyhow::Result<()> {
        let cfg = config::Config::resolve(
            self.network.as_deref(),
            self.wallet_name.as_deref(),
            self.wallet_path.as_deref(),
        )?;

        match self.command {
            Command::Wallet { command } => command.execute(&cfg).await,
            Command::Stake { command } => command.execute(&cfg).await,
            Command::Transfer { command } => command.execute(&cfg).await,
            Command::Register { command } => command.execute(&cfg).await,
            Command::Root { command } => command.execute(&cfg).await,
            Command::Subnet { command } => command.execute(&cfg).await,
            Command::Delegate { command } => command.execute(&cfg).await,
            Command::Weights { command } => command.execute(&cfg).await,
            Command::Metagraph { command } => command.execute(&cfg).await,
            #[cfg(feature = "mev")]
            Command::Mev { command } => command.execute(&cfg).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_wallet_create() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "create"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Create { .. } } => {}
            other => panic!("expected Wallet::Create, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_list() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "list"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::List } => {}
            other => panic!("expected Wallet::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_show() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "show"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Show { .. } } => {}
            other => panic!("expected Wallet::Show, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_balance() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "balance", "--all"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Balance { all, .. } } => {
                assert!(all);
            }
            other => panic!("expected Wallet::Balance, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_overview() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "overview"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Overview { .. } } => {}
            other => panic!("expected Wallet::Overview, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_transfer() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "wallet", "transfer", "5Dest...", "1.5"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Transfer { dest, amount, password: _ } } => {
                assert_eq!(dest, "5Dest...");
                assert_eq!(amount, "1.5");
            }
            other => panic!("expected Wallet::Transfer, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_swap_coldkey() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "wallet", "swap-coldkey", "5NewColdkey..."]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::SwapColdkey { new_coldkey, .. } } => {
                assert_eq!(new_coldkey, "5NewColdkey...");
            }
            other => panic!("expected Wallet::SwapColdkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_inspect() {
        let cli = Cli::try_parse_from(["btcli-rs", "wallet", "inspect"]).unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::Inspect { .. } } => {}
            other => panic!("expected Wallet::Inspect, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_coldkey() {
        let cli = Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "regen-coldkey",
            "word1 word2 word3",
            "--yes",
        ])
        .unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::RegenColdkey { mnemonic, yes, .. } } => {
                assert_eq!(mnemonic, "word1 word2 word3");
                assert!(yes);
            }
            other => panic!("expected Wallet::RegenColdkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_coldkeypub() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "wallet", "regen-coldkeypub", "5ColdkeyPubAddr..."])
                .unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::RegenColdkeypub { ss58_address } } => {
                assert_eq!(ss58_address, "5ColdkeyPubAddr...");
            }
            other => panic!("expected Wallet::RegenColdkeypub, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_create_hotkey() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "wallet", "create-hotkey", "--hotkey", "validator"])
                .unwrap();
        match cli.command {
            Command::Wallet {
                command: WalletCommand::CreateHotkey { hotkey, password: _, seed },
            } => {
                assert_eq!(hotkey, "validator");
                assert!(!seed);
            }
            other => panic!("expected Wallet::CreateHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_create_hotkey_seed() {
        let cli = Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "create-hotkey",
            "--hotkey",
            "miner",
            "--seed",
        ])
        .unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::CreateHotkey { seed, .. } } => {
                assert!(seed);
            }
            other => panic!("expected Wallet::CreateHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_hotkey() {
        let cli = Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "regen-hotkey",
            "word1 word2 word3",
            "--hotkey",
            "my-hotkey",
        ])
        .unwrap();
        match cli.command {
            Command::Wallet { command: WalletCommand::RegenHotkey { mnemonic, hotkey } } => {
                assert_eq!(mnemonic, "word1 word2 word3");
                assert_eq!(hotkey, "my-hotkey");
            }
            other => panic!("expected Wallet::RegenHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_network_flag() {
        let cli = Cli::try_parse_from(["btcli-rs", "--network", "test", "wallet", "list"]).unwrap();
        assert_eq!(cli.network.as_deref(), Some("test"));
    }

    #[test]
    fn parse_wallet_name_flag() {
        let cli = Cli::try_parse_from(["btcli-rs", "--wallet.name", "my-wallet", "wallet", "list"])
            .unwrap();
        assert_eq!(cli.wallet_name.as_deref(), Some("my-wallet"));
    }

    #[test]
    fn parse_wallet_path_flag() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "--wallet.path", "/tmp/wallets", "wallet", "list"])
                .unwrap();
        assert_eq!(cli.wallet_path.as_deref(), Some("/tmp/wallets"));
    }

    #[test]
    fn parse_no_subcommand_fails() {
        let result = Cli::try_parse_from(["btcli-rs"]);
        assert!(result.is_err());
    }

    #[test]
    fn help_output_contains_btcli_rs() {
        let result = Cli::try_parse_from(["btcli-rs", "--help"]);
        assert!(result.is_err()); // --help causes early exit
        let err = result.unwrap_err();
        let output = err.to_string();
        assert!(output.contains("btcli-rs"), "help should contain btcli-rs");
    }

    // --- New top-level command parsing tests ---

    #[test]
    fn parse_stake_add() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "stake", "add", "--hotkey", "5HK", "1.0"]).unwrap();
        match cli.command {
            Command::Stake { command: StakeCommand::Add { hotkey, amount, .. } } => {
                assert_eq!(hotkey, "5HK");
                assert_eq!(amount, "1.0");
            }
            other => panic!("expected Stake::Add, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_remove() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "stake", "remove", "--hotkey", "5HK", "2.0"]).unwrap();
        match cli.command {
            Command::Stake { command: StakeCommand::Remove { hotkey, amount, .. } } => {
                assert_eq!(hotkey, "5HK");
                assert_eq!(amount, "2.0");
            }
            other => panic!("expected Stake::Remove, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_list() {
        let cli = Cli::try_parse_from(["btcli-rs", "stake", "list"]).unwrap();
        match cli.command {
            Command::Stake { command: StakeCommand::List } => {}
            other => panic!("expected Stake::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_transfer_top_level() {
        let cli = Cli::try_parse_from([
            "btcli-rs",
            "transfer",
            "transfer",
            "5Dest",
            "5.0",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            Command::Transfer { command: TransferCommand::Transfer { dest, amount, password } } => {
                assert_eq!(dest, "5Dest");
                assert_eq!(amount, "5.0");
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Transfer::Transfer, got {other:?}"),
        }
    }

    #[test]
    fn parse_transfer_multiple() {
        let cli = Cli::try_parse_from([
            "btcli-rs",
            "transfer",
            "multiple",
            "--destinations",
            "5A,5B",
            "--amounts",
            "1.0,2.0",
        ])
        .unwrap();
        match cli.command {
            Command::Transfer {
                command: TransferCommand::Multiple { destinations, amounts, .. },
            } => {
                assert_eq!(destinations, "5A,5B");
                assert_eq!(amounts, "1.0,2.0");
            }
            other => panic!("expected Transfer::Multiple, got {other:?}"),
        }
    }

    #[test]
    fn parse_register_pow() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "register", "register", "--netuid", "3"]).unwrap();
        match cli.command {
            Command::Register { command: RegistrationCommand::Register { netuid, .. } } => {
                assert_eq!(netuid, 3);
            }
            other => panic!("expected Register::Register, got {other:?}"),
        }
    }

    #[test]
    fn parse_register_burned() {
        let cli = Cli::try_parse_from(["btcli-rs", "register", "burned-register", "--netuid", "7"])
            .unwrap();
        match cli.command {
            Command::Register { command: RegistrationCommand::BurnedRegister { netuid, .. } } => {
                assert_eq!(netuid, 7);
            }
            other => panic!("expected Register::BurnedRegister, got {other:?}"),
        }
    }

    #[test]
    fn parse_root_register_top_level() {
        let cli =
            Cli::try_parse_from(["btcli-rs", "root", "register", "--password", "pw4"]).unwrap();
        match cli.command {
            Command::Root { command: RootCommand::Register { password } } => {
                assert_eq!(password.unwrap(), "pw4");
            }
            other => panic!("expected Root::Register, got {other:?}"),
        }
    }

    #[test]
    fn all_top_level_commands_parseable() {
        let commands: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "wallet", "create"],
            vec!["btcli-rs", "stake", "list"],
            vec!["btcli-rs", "transfer", "transfer", "5D", "1.0"],
            vec!["btcli-rs", "register", "register", "--netuid", "1"],
            vec!["btcli-rs", "root", "register"],
            vec!["btcli-rs", "subnet", "list"],
            vec!["btcli-rs", "delegate", "list"],
            vec!["btcli-rs", "weights", "set-weights", "--netuid", "1", "1,2", "100,200", "--yes"],
            vec!["btcli-rs", "weights", "get-weights", "--netuid", "1"],
            vec!["btcli-rs", "metagraph", "show", "--netuid", "1"],
            vec!["btcli-rs", "metagraph", "sync", "--netuid", "1"],
            vec!["btcli-rs", "root", "set-weights", "--netuid", "1", "1,2", "100,200"],
            vec!["btcli-rs", "root", "get-weights", "--netuid", "1", "5"],
            vec!["btcli-rs", "root", "claim", "1,3"],
        ];
        for args in &commands {
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "command {:?} should be parseable", args);
        }
    }
}
