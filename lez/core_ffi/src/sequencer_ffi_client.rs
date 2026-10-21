use std::{collections::BTreeMap, fmt::Display};

use anyhow::Result;
use lee::Actor;
use lee_core::CommitmentPosition;
use sequencer_executor_actor::protocol::{
    GetAccount, GetAccountBalance, GetAccountNonces, GetAccountView, GetBlock, GetBlockRange,
    GetLastBlockId, GetProofsAndRoot, GetPublications, GetRecoveryBinding, GetTransaction,
    Transaction, TransactionOrigin,
};
use wallet::{ExecutionFailureKind, WalletSequencerClient};

use crate::sequencer::SequencerServiceFFI;

#[derive(Clone)]
pub struct SequencerServiceFFIWrapper(pub &'static SequencerServiceFFI);

impl SequencerServiceFFIWrapper {
    #[must_use]
    pub const fn wrap(sequencer: &'static SequencerServiceFFI) -> Self {
        Self(sequencer)
    }
}

#[derive(Debug, Clone)]
pub struct FfiSequencerClientError(pub String);

impl Display for FfiSequencerClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sequencer FFI client error : {:?}", self.0)
    }
}

impl std::error::Error for FfiSequencerClientError {}

impl From<anyhow::Error> for FfiSequencerClientError {
    fn from(value: anyhow::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<FfiSequencerClientError> for sequencer_service_rpc::ClientError {
    fn from(value: FfiSequencerClientError) -> Self {
        Self::Custom(value.0)
    }
}

impl From<FfiSequencerClientError> for ExecutionFailureKind {
    fn from(value: FfiSequencerClientError) -> Self {
        sequencer_service_rpc::ClientError::from(value).into()
    }
}

impl WalletSequencerClient for SequencerServiceFFIWrapper {
    type Error = FfiSequencerClientError;

    fn from_config(_config: &wallet::config::WalletConfig) -> Result<Self> {
        anyhow::bail!(
            "Wallet config des not hold enough information to produce sequencer ffi, use corresponding sequencer_ffi_* function"
        );
    }

    async fn get_account(
        &self,
        account_id: lee::AccountId,
    ) -> Result<Option<lee::Account>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccount { account_id })
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .account)
    }

    async fn get_account_balance(&self, account_id: lee::AccountId) -> Result<u128, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountBalance { account_id })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_accounts_nonces(
        &self,
        account_ids: Vec<lee::AccountId>,
    ) -> Result<Vec<lee_core::account::Nonce>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountNonces { account_ids })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_block(
        &self,
        block_id: lee_core::BlockId,
    ) -> Result<Option<common::block::Block>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetBlock { block_id })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_block_range(
        &self,
        start_block_id: lee_core::BlockId,
        end_block_id: lee_core::BlockId,
    ) -> Result<Vec<common::block::Block>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetBlockRange {
                range: (start_block_id..=end_block_id)
                    .try_into()
                    .map_err(anyhow::Error::from)?,
            })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_last_block_id(&self) -> Result<lee_core::BlockId, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetLastBlockId {})
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_transaction(
        &self,
        tx_hash: common::HashType,
    ) -> Result<Option<(common::transaction::LeeTransaction, lee_core::BlockId)>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetTransaction { tx_hash })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn send_transaction(
        &self,
        tx: common::transaction::LeeTransaction,
    ) -> Result<common::HashType, Self::Error> {
        let tx_hash = tx.hash();

        self.0
            .executor_ref()
            .ask(Transaction {
                transaction: tx,
                origin: TransactionOrigin::User,
            })
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        Ok(tx_hash)
    }

    async fn get_account_view(
        &self,
        actor_state_selector: Actor,
    ) -> Result<Option<lee::Account>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetAccountView {
                actor_state_selector,
            })
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .account)
    }

    async fn get_program_ids(
        &self,
    ) -> Result<std::collections::BTreeMap<String, lee::ProgramId>, Self::Error> {
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
    ) -> Result<
        (
            Vec<Option<lee_core::MembershipProof>>,
            Option<lee_core::MembershipProof>,
            lee_core::CommitmentSetDigest,
        ),
        Self::Error,
    > {
        Ok(self
            .0
            .executor_ref()
            .ask(GetProofsAndRoot {
                commitments,
                message_position,
            })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_publications(
        &self,
        from_position: CommitmentPosition,
        limit: common::PageLimit,
    ) -> Result<Vec<(CommitmentPosition, lee::Publication)>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetPublications {
                from_position,
                limit,
            })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }

    async fn get_recovery_binding(
        &self,
        address: lee::AccountId,
    ) -> Result<Option<lee::EncryptedNote>, Self::Error> {
        Ok(self
            .0
            .executor_ref()
            .ask(GetRecoveryBinding { address })
            .send()
            .await
            .map_err(anyhow::Error::from)?)
    }
}
