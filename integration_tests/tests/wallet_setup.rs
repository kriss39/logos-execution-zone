#![expect(
    clippy::tests_outside_test_module,
    reason = "We don't care about these in tests"
)]

use std::time::Duration;

use anyhow::{Context as _, Result};
use sequencer_service_rpc::RpcClient as _;
use test_fixtures::TestContext;
use tokio::test;
use wallet::{WalletCore, config::WalletConfigOverrides};

/// A fresh seed owns nothing earlier, so setup starts at the tip.
#[test]
async fn fresh_wallet_starts_at_chain_tip() -> Result<()> {
    let ctx = TestContext::new().await?;

    let mut tip_before = ctx.sequencer_client().get_last_block_id().await?;
    for _ in 0..30_u32 {
        if tip_before > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        tip_before = ctx.sequencer_client().get_last_block_id().await?;
    }
    anyhow::ensure!(tip_before > 0, "Chain did not produce a block");

    let temp_dir = tempfile::tempdir().context("Failed to create temp dir for wallet home")?;
    let overrides = WalletConfigOverrides {
        sequencers: Some(ctx.wallet().config().sequencers.clone()),
        multi_sequencer_client_config: Some(
            ctx.wallet().config().multi_sequencer_client_config.clone(),
        ),
        ..WalletConfigOverrides::default()
    };

    let (wallet, _mnemonic) = WalletCore::new_init_storage(
        temp_dir.path().join("wallet_config.json"),
        temp_dir.path().join("storage.json"),
        temp_dir.path().join("metrics.json"),
        Some(overrides.clone()),
        "test_pass",
    )
    .await?;

    let tip_after = ctx.sequencer_client().get_last_block_id().await?;
    let cursor = wallet.storage().last_synced_block();
    wallet.store_persistent_data()?;
    // Close the wallet's clients before the node goes away.
    drop(wallet);

    assert!(
        cursor >= tip_before && cursor <= tip_after,
        "Fresh wallet cursor {cursor} is outside the tip range {tip_before}..={tip_after}"
    );

    let reopened = WalletCore::new_update_chain(
        temp_dir.path().join("wallet_config.json"),
        temp_dir.path().join("storage.json"),
        temp_dir.path().join("metrics.json"),
        Some(overrides),
    )
    .await?;
    let reopened_cursor = reopened.storage().last_synced_block();
    drop(reopened);

    assert_eq!(
        reopened_cursor, cursor,
        "Cursor did not survive a save and reopen"
    );

    Ok(())
}
