//! `system_upgrader`: upgrades and installs the system programs, each one of its PDAs
//! (`PdaSeed::for_system_program(name)`). Pending upgrades live in its registry.

use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
pub use lee_core::program::{SYSTEM_UPGRADER_ACCOUNT_ID, SystemProgramName};
use lee_core::{
    BlockId,
    account::{AccountId, Actor},
    program::{PdaSeed, ProgramId},
};

const REGISTRY_SEED: [u8; 32] = *b"/LEZ/v0.3/SystemRegistry/0000000";

/// A pending upgrade, stored in the [`Registry`] under the system program's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ScheduledUpgrade {
    pub first_segment: AccountId,
    /// What the chain at `first_segment` computes to, checked by the loader at `Schedule`.
    pub image_id: ProgramId,
    pub activation_height: BlockId,
}

/// Every upgradable system program, by name, with its pending upgrade, if any. Held in the
/// registry's actor state ([`registry_actor`]): seeded at genesis, extended by `Install`.
#[derive(Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Registry {
    pub programs: BTreeMap<SystemProgramName, Option<ScheduledUpgrade>>,
}

impl Registry {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("a registry serializes")
    }

    /// An empty registry for an empty actor state.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return Some(Self::default());
        }
        borsh::from_slice(bytes).ok()
    }

    /// `name`'s pending upgrade; `None` if nothing is pending or `name` isn't registered.
    #[must_use]
    pub fn scheduled(&self, name: &SystemProgramName) -> Option<ScheduledUpgrade> {
        self.programs.get(name).copied().flatten()
    }
}

/// Where the [`Registry`] lives: a PDA of `system_upgrader`.
#[must_use]
pub fn registry_account_id() -> AccountId {
    AccountId::for_public_pda(&SYSTEM_UPGRADER_ACCOUNT_ID, &PdaSeed::new(REGISTRY_SEED))
}

/// The actor every `system_upgrader` message is sent to: the registry, under `system_upgrader`.
#[must_use]
pub fn registry_actor() -> Actor {
    Actor::new(registry_account_id(), SYSTEM_UPGRADER_ACCOUNT_ID)
}

/// `system_upgrader`'s fixed address.
#[must_use]
pub const fn system_upgrader_account_id() -> AccountId {
    SYSTEM_UPGRADER_ACCOUNT_ID
}
