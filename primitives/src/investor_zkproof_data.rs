use crate::{IdentityId, InvestorUid, Ticker};
use cryptography::claim_proofs::{
    build_scope_claim_proof_data, CddClaimData, ProofKeyPair, ScopeClaimData,
};

use blake2::{Blake2s, Digest};
use schnorrkel::Signature;

use codec::{Decode, Encode, Error as CodecError, Input, Output};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// It is a wrapper over the signature generated by `ProofKeyPair::generate_id_match_proof`.
///
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InvestorZKProofData(pub Signature);

impl InvestorZKProofData {
    /// Creates the proof.
    pub fn new(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        // Create CDD Claim and Scope claim.
        let cdd_claim = Self::make_cdd_claim(did, investor);
        let scope_claim = Self::make_scope_claim(ticker.as_slice(), investor);

        // Create the scope claim proof
        let sc_proof = build_scope_claim_proof_data(&cdd_claim, &scope_claim);
        let pair = ProofKeyPair::from(sc_proof);

        // Generate the message and its ID match proof.
        let message = Self::make_message(did, ticker.as_slice());
        let proof = pair.generate_id_match_proof(&message[..]);

        Self(proof)
    }

    /// Returns the CDD claim of the given `investor_did` and `investor_uid`.
    pub fn make_cdd_claim(
        investor_did: &IdentityId,
        investor_unique_id: &InvestorUid,
    ) -> CddClaimData {
        CddClaimData::new(&investor_did.to_bytes(), &investor_unique_id.to_bytes())
    }

    /// Returns the Scope claim of the given `ticker` and `investor_uid`.
    pub fn make_scope_claim(scope: &[u8], investor_unique_id: &InvestorUid) -> ScopeClaimData {
        ScopeClaimData::new(scope, &investor_unique_id.to_bytes())
    }

    /// Returns the message used for testing the proof.
    pub fn make_message(investor_did: &IdentityId, scope: &[u8]) -> [u8; 32] {
        Blake2s::default()
            .chain(investor_did)
            .chain(scope)
            .finalize()
            .into()
    }
}

impl Encode for InvestorZKProofData {
    #[inline]
    fn size_hint(&self) -> usize {
        64
    }

    fn encode_to<W: Output>(&self, dest: &mut W) {
        let signature = self.0.to_bytes();
        signature.encode_to(dest);
    }
}

impl Decode for InvestorZKProofData {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let signature_data = <[u8; 64]>::decode(input)?;
        let signature = Signature::from_bytes(&signature_data)
            .map_err(|_| CodecError::from("Invalid ProofData"))?;

        Ok(InvestorZKProofData(signature))
    }
}
