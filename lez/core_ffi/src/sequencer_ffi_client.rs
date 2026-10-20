use std::collections::BTreeMap;

use anyhow::Result;
use lee::Actor;
use lee_core::CommitmentPosition;
use sequencer_executor_actor::protocol::{
    GetAccount, GetAccountBalance, GetAccountNonces, GetAccountView, GetBlock, GetBlockRange,
    GetLastBlockId, GetProofsAndRoot, GetPublications, GetRecoveryBinding, GetTransaction,
    Transaction, TransactionOrigin,
};
use wallet::WalletSequencerClient;

use crate::sequencer::SequencerServiceFFI;

#[derive(Clone)]
pub struct SequencerServiceFFIWrapper(pub &'static SequencerServiceFFI);

impl SequencerServiceFFIWrapper {
    #[must_use]
    pub const fn wrap(sequencer: &'static SequencerServiceFFI) -> Self {
        Self(sequencer)
    }
}

impl WalletSequencerClient for SequencerServiceFFIWrapper {
    fn from_config(_config: &wallet::config::WalletConfig) -> Result<Self> {
        anyhow::bail!(
            "Wallet config des not hold enough information to produce sequencer ffi, use corresponding sequencer_ffi_* function"
        );
    }

    async fn get_account(&self, account_id: lee::AccountId) -> Result<Option<lee::Account>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccount { account_id })
            .send()
            .await?
            .account)
    }

    async fn get_account_balance(&self, account_id: lee::AccountId) -> Result<u128> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountBalance { account_id })
            .send()
            .await?)
    }

    async fn get_accounts_nonces(
        &self,
        account_ids: Vec<lee::AccountId>,
    ) -> Result<Vec<lee_core::account::Nonce>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountNonces { account_ids })
            .send()
            .await?)
    }

    async fn get_block(&self, block_id: lee_core::BlockId) -> Result<Option<common::block::Block>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetBlock { block_id })
            .send()
            .await?)
    }

    async fn get_block_range(
        &self,
        start_block_id: lee_core::BlockId,
        end_block_id: lee_core::BlockId,
    ) -> Result<Vec<common::block::Block>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetBlockRange {
                range: (start_block_id..=end_block_id).try_into()?,
            })
            .send()
            .await?)
    }

    async fn get_last_block_id(&self) -> Result<lee_core::BlockId> {
        Ok(self.0.executor_ref().ask(GetLastBlockId {}).send().await?)
    }

    async fn get_transaction(
        &self,
        tx_hash: common::HashType,
    ) -> Result<Option<(common::transaction::LeeTransaction, lee_core::BlockId)>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetTransaction { tx_hash })
            .send()
            .await?)
    }

    async fn send_transaction(
        &self,
        tx: common::transaction::LeeTransaction,
    ) -> Result<common::HashType> {
        let tx_hash = tx.hash();

        self.0
            .executor_ref()
            .ask(Transaction {
                transaction: tx,
                origin: TransactionOrigin::User,
            })
            .send()
            .await?;

        Ok(tx_hash)
    }

    async fn get_account_view(&self, actor_state_selector: Actor) -> Result<Option<lee::Account>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountView {
                actor_state_selector,
            })
            .send()
            .await?
            .account)
    }

    async fn get_program_ids(&self) -> Result<std::collections::BTreeMap<String, lee::ProgramId>> {
        // TODO: Sequencer RPC don't read programs from state. Need to update both.
        let mut program_ids = BTreeMap::new();
        program_ids.insert(
            "privacy_preserving_circuit".to_owned(),
            lee::PRIVACY_PRESERVING_CIRCUIT_ID,
        );
        Ok(program_ids)
    }

    async fn get_proofs_and_root(
        &self,
        commitments: Vec<lee_core::Commitment>,
        message_position: Option<CommitmentPosition>,
    ) -> Result<(
        Vec<Option<lee_core::MembershipProof>>,
        Option<lee_core::MembershipProof>,
        lee_core::CommitmentSetDigest,
    )> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetProofsAndRoot {
                commitments,
                message_position,
            })
            .send()
            .await?)
    }

    async fn get_publications(
        &self,
        from_position: CommitmentPosition,
        limit: common::PageLimit,
    ) -> Result<Vec<(CommitmentPosition, lee::Publication)>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetPublications {
                from_position,
                limit,
            })
            .send()
            .await?)
    }

    async fn get_recovery_binding(
        &self,
        address: lee::AccountId,
    ) -> Result<Option<lee::EncryptedNote>> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetRecoveryBinding { address })
            .send()
            .await?)
    }
}
