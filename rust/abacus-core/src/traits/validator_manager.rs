use std::fmt::Debug;

use async_trait::async_trait;
use eyre::Result;

use crate::{
    traits::{ChainCommunicationError, TxOutcome},
    MultisigSignedCheckpoint,
};

/// Interface for an InboxValidatorManager
#[async_trait]
pub trait InboxValidatorManager: Send + Sync + Debug {
    /// Submit a signed checkpoint for inclusion
    /// Mocks already have a function called checkpoint
    async fn submit_checkpoint(
        &self,
        multisig_signed_checkpoint: &MultisigSignedCheckpoint,
    ) -> Result<TxOutcome, ChainCommunicationError>;
}