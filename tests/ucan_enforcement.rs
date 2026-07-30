use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;

use basalt::authority_core::default_authority_policy_ref;
use basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_DIGEST_BYTES;
use basalt::ucan::token::DenyUnknownCaveats;
use basalt::ucan::{
    AudienceDid, CapabilityDocument, CapabilitySet, CaveatDecision, CaveatDocument,
    CaveatIdentifier, CaveatPolicy, CaveatPolicySet, CompactToken, ED25519_SECRET_KEY_BYTES,
    Ed25519InMemorySigner, Ed25519PublicKey, HolderInvocationLimits, HolderInvocationNonce,
    HolderInvocationReplayAdmission, HolderInvocationReplayCandidate, HolderInvocationReplayError,
    HolderInvocationSigningRequest, IssuerDid, KeyResolutionContext, KeyResolver, KeyResolverError,
    NoRevocations, OperationBinding, OperationProfileId, ProofCollection, ProofReferences,
    ProofStore, RevocationChecker, RevocationList, SignerSignature, TokenSigner, TokenTimeBounds,
    VerificationLimits, VerificationTime, issue_token_with_signer, proof_reference,
    sign_holder_invocation,
};
use basalt::ucan_enforcement::{
    EnforcementAction, LiveUcanEnforcement, UcanEnforcementBoundaries, UcanEnforcementFailure,
    UcanEnforcementReason, UcanEnforcementReceipt, UcanEnforcementRequest, bind_enforcement_action,
    verify_ucan_enforcement_request,
};
use basalt::{CapabilityGrant, EnforcementRequest, default_policy};

const ISSUER_SEED_BYTE: u8 = 11;
const DELEGATE_SEED_BYTE: u8 = 13;
const HOLDER_SEED_BYTE: u8 = 17;
const WRONG_ISSUER_SEED_BYTE: u8 = 31;
const NONCE_BYTE: u8 = 19;
const OTHER_NONCE_BYTE: u8 = 23;
const TOKEN_NOT_BEFORE_SECONDS: u64 = 1_000;
const TOKEN_EXPIRES_SECONDS: u64 = 2_000;
const VERIFY_AT_SECONDS: u64 = 1_500;
const EXPIRED_VERIFY_AT_SECONDS: u64 = 2_500;
const PARENT_RESOURCE: &str = "did:key:z6Mkresource";
const DOCUMENT_RESOURCE: &str = "did:key:z6Mkresource/docs/report";
const PRIVATE_RESOURCE: &str = "did:key:z6Mkresource/private/report";
const OTHER_RESOURCE: &str = "did:key:z6Mkresource/videos/report";
const SUBSTITUTED_REQUEST_RESOURCE: &str = "did:key:z6Mkresource/docs/other";
const DOCUMENT_WRITE_ABILITY: &str = "document/write";
const DOCUMENT_READ_ABILITY: &str = "document/read";
const CAVEAT_DOMAIN: &str = "example";
const CAVEAT_TYPE: &str = "approval";
const CAVEAT_KEY: &str = "required";
const CAVEAT_PAYLOAD: &[u8] = b"yes";
const BACKEND_ERROR_MESSAGE: &str = "injected resolver failure";
const EXPECTED_PREFLIGHT_RESOLVER_CALLS: u32 = 1;
const EXPECTED_CAVEAT_OBSERVATIONS: u32 = 2;
const FIRST_CALL: u32 = 1;
const OVERSIZED_TOKEN_LIMIT: usize = 64;
const OVERSIZED_TOKEN_BYTES: usize = OVERSIZED_TOKEN_LIMIT + 1;
const OVERSIZED_REQUEST_BYTES: usize = 4_097;
const SHORT_CANONICAL_RESOURCE_LIMIT: usize = 8;
const UNKNOWN_PROFILE_BYTE: u8 = 29;

struct InvocationFixture {
    token: CompactToken,
    holder: Ed25519InMemorySigner,
    audience: AudienceDid,
    keys: KeyResolutionContext,
    proofs: ProofCollection,
}

impl InvocationFixture {
    fn direct(capability: CapabilityDocument) -> Self {
        Self::direct_with_expiration(capability, TOKEN_EXPIRES_SECONDS)
    }

    fn direct_with_expiration(capability: CapabilityDocument, expires_at_seconds: u64) -> Self {
        let issuer = signer(ISSUER_SEED_BYTE);
        let holder = signer(HOLDER_SEED_BYTE);
        let audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
        let token = issue_token_with_signer(
            &issuer,
            &audience,
            &CapabilitySet::new(vec![capability]).expect("test capability must validate"),
            &ProofReferences::empty(),
            TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, expires_at_seconds)
                .expect("test time bounds must validate"),
        )
        .expect("direct test token must issue");
        let keys = KeyResolutionContext::new(vec![
            issuer
                .verification_key()
                .expect("issuer verification key must derive"),
            holder
                .verification_key()
                .expect("holder verification key must derive"),
        ]);
        Self {
            token,
            holder,
            audience,
            keys,
            proofs: ProofCollection::empty(),
        }
    }

    fn delegated() -> Self {
        let issuer = signer(ISSUER_SEED_BYTE);
        let delegate = signer(DELEGATE_SEED_BYTE);
        let holder = signer(HOLDER_SEED_BYTE);
        let delegate_audience =
            AudienceDid::from(delegate.issuer().expect("delegate DID must derive"));
        let audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
        let parent = issue_token_with_signer(
            &issuer,
            &delegate_audience,
            &CapabilitySet::new(vec![document_capability(PARENT_RESOURCE)])
                .expect("parent capability must validate"),
            &ProofReferences::empty(),
            test_time_bounds(),
        )
        .expect("parent token must issue");
        let parent_reference = proof_reference(&parent);
        let child = issue_token_with_signer(
            &delegate,
            &audience,
            &CapabilitySet::new(vec![document_capability(DOCUMENT_RESOURCE)])
                .expect("child capability must validate"),
            &ProofReferences::from_references(vec![parent_reference]),
            test_time_bounds(),
        )
        .expect("child token must issue");
        let keys = KeyResolutionContext::new(vec![
            issuer
                .verification_key()
                .expect("issuer verification key must derive"),
            delegate
                .verification_key()
                .expect("delegate verification key must derive"),
            holder
                .verification_key()
                .expect("holder verification key must derive"),
        ]);
        Self {
            token: child,
            holder,
            audience,
            keys,
            proofs: ProofCollection::from_tokens(vec![parent]),
        }
    }
}

struct TestServices<'a> {
    key_resolver: &'a dyn KeyResolver,
    proof_store: &'a dyn ProofStore,
    revocations: &'a dyn RevocationChecker,
    caveats: &'a dyn CaveatPolicySet,
    replay: &'a dyn HolderInvocationReplayAdmission,
    verification_time: VerificationTime,
    verification_limits: VerificationLimits,
    holder_limits: HolderInvocationLimits,
}

struct RecordingReplay<'a> {
    calls: Cell<u32>,
    seen: RefCell<
        BTreeSet<(
            String,
            [u8; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES],
        )>,
    >,
    caveat_calls: Option<&'a Cell<u32>>,
    expected_caveat_calls: u32,
    committed: Cell<bool>,
}

impl RecordingReplay<'_> {
    const fn new() -> Self {
        Self {
            calls: Cell::new(0),
            seen: RefCell::new(BTreeSet::new()),
            caveat_calls: None,
            expected_caveat_calls: 0,
            committed: Cell::new(false),
        }
    }

    const fn after_caveats(
        caveat_calls: &Cell<u32>,
        expected_caveat_calls: u32,
    ) -> RecordingReplay<'_> {
        RecordingReplay {
            calls: Cell::new(0),
            seen: RefCell::new(BTreeSet::new()),
            caveat_calls: Some(caveat_calls),
            expected_caveat_calls,
            committed: Cell::new(false),
        }
    }
}

impl HolderInvocationReplayAdmission for RecordingReplay<'_> {
    fn admit_holder_invocation(
        &self,
        candidate: &HolderInvocationReplayCandidate<'_>,
    ) -> Result<(), HolderInvocationReplayError> {
        self.calls.set(
            self.calls
                .get()
                .checked_add(FIRST_CALL)
                .expect("test replay call count must not overflow"),
        );
        if let Some(caveat_calls) = self.caveat_calls {
            assert_eq!(caveat_calls.get(), self.expected_caveat_calls);
        }
        let replay_key = (
            candidate.audience().as_str().to_owned(),
            *candidate.nonce().as_bytes(),
        );
        if !self.seen.borrow_mut().insert(replay_key) {
            return Err(HolderInvocationReplayError::Duplicate);
        }
        self.committed.set(true);
        Ok(())
    }
}

struct CountingCaveatPolicy {
    calls: Cell<u32>,
    is_allowed: bool,
}

impl CountingCaveatPolicy {
    const fn new(is_allowed: bool) -> Self {
        Self {
            calls: Cell::new(0),
            is_allowed,
        }
    }
}

impl CaveatPolicy for CountingCaveatPolicy {
    fn evaluate(
        &self,
        _caveat: &CaveatDocument,
        _request: &basalt::ucan::AuthorizationRequest,
    ) -> CaveatDecision {
        self.calls.set(
            self.calls
                .get()
                .checked_add(FIRST_CALL)
                .expect("test caveat call count must not overflow"),
        );
        if self.is_allowed {
            CaveatDecision::Satisfied
        } else {
            CaveatDecision::Rejected {
                message: "test caveat denied".to_owned(),
            }
        }
    }
}

struct CountingCaveatSet {
    policy: CountingCaveatPolicy,
}

impl CountingCaveatSet {
    const fn new(is_allowed: bool) -> Self {
        Self {
            policy: CountingCaveatPolicy::new(is_allowed),
        }
    }
}

impl CaveatPolicySet for CountingCaveatSet {
    fn policy_for(&self, _caveat: &CaveatIdentifier) -> Option<&dyn CaveatPolicy> {
        Some(&self.policy)
    }
}

struct WrongIssuerKeyResolver {
    public_key: Ed25519PublicKey,
}

impl KeyResolver for WrongIssuerKeyResolver {
    fn resolve_key(&self, _issuer: &IssuerDid) -> Result<Ed25519PublicKey, KeyResolverError> {
        Ok(self.public_key.clone())
    }
}

struct FailsAfterResolver<'a> {
    inner: &'a KeyResolutionContext,
    allowed_calls: u32,
    calls: Cell<u32>,
}

impl KeyResolver for FailsAfterResolver<'_> {
    fn resolve_key(&self, issuer: &IssuerDid) -> Result<Ed25519PublicKey, KeyResolverError> {
        let next_call = self
            .calls
            .get()
            .checked_add(FIRST_CALL)
            .expect("test resolver call count must not overflow");
        self.calls.set(next_call);
        if next_call > self.allowed_calls {
            return Err(KeyResolverError::Backend {
                issuer: issuer.as_str().to_owned(),
                message: BACKEND_ERROR_MESSAGE.to_owned(),
            });
        }
        self.inner.resolve_key(issuer)
    }
}

fn signer(seed_byte: u8) -> Ed25519InMemorySigner {
    Ed25519InMemorySigner::from_seed_bytes([seed_byte; ED25519_SECRET_KEY_BYTES])
}

fn test_time_bounds() -> TokenTimeBounds {
    TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS)
        .expect("test time bounds must validate")
}

fn document_capability(resource: &str) -> CapabilityDocument {
    CapabilityDocument::new(resource.to_owned(), DOCUMENT_WRITE_ABILITY.to_owned())
        .expect("test capability must validate")
}

fn caveated_document_capability() -> CapabilityDocument {
    let caveat = CaveatDocument::new(
        CAVEAT_DOMAIN.to_owned(),
        CAVEAT_TYPE.to_owned(),
        CAVEAT_KEY.to_owned(),
        CAVEAT_PAYLOAD.to_vec(),
    )
    .expect("test caveat must validate");
    CapabilityDocument::with_caveats(
        PARENT_RESOURCE.to_owned(),
        DOCUMENT_WRITE_ABILITY.to_owned(),
        vec![caveat],
    )
    .expect("caveated test capability must validate")
}

fn enforcement_request(resource: &str) -> EnforcementRequest {
    EnforcementRequest::new(
        "document-write",
        resource.to_owned(),
        DOCUMENT_WRITE_ABILITY,
    )
}

const fn nonce() -> HolderInvocationNonce {
    HolderInvocationNonce::from_bytes(
        [NONCE_BYTE; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES],
    )
}

const fn other_nonce() -> HolderInvocationNonce {
    HolderInvocationNonce::from_bytes(
        [OTHER_NONCE_BYTE; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES],
    )
}

fn sign_request(
    fixture: &InvocationFixture,
    request: &EnforcementRequest,
    request_nonce: HolderInvocationNonce,
) -> SignerSignature {
    let operation = bind_enforcement_action(EnforcementAction::DocumentWrite, request)
        .expect("test request must map to a typed operation");
    sign_operation(fixture, request, request_nonce, operation)
}

fn sign_operation(
    fixture: &InvocationFixture,
    request: &EnforcementRequest,
    request_nonce: HolderInvocationNonce,
    operation: OperationBinding,
) -> SignerSignature {
    sign_holder_invocation(
        &HolderInvocationSigningRequest::new(
            fixture.token.as_str(),
            &fixture.audience,
            request.resource(),
            request.ability(),
            operation,
            request_nonce,
        ),
        &fixture.holder,
        VerificationLimits::default(),
        HolderInvocationLimits::default(),
    )
    .expect("test holder request must sign")
}

fn services<'a>(
    fixture: &'a InvocationFixture,
    revocations: &'a dyn RevocationChecker,
    caveats: &'a dyn CaveatPolicySet,
    replay: &'a dyn HolderInvocationReplayAdmission,
) -> TestServices<'a> {
    TestServices {
        key_resolver: &fixture.keys,
        proof_store: &fixture.proofs,
        revocations,
        caveats,
        replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    }
}

// Own the time and limit snapshot used by one adapter evaluation.
#[allow(clippy::needless_pass_by_value)]
fn evaluate<'request>(
    compact_token: &str,
    request: &'request EnforcementRequest,
    signature: &SignerSignature,
    request_nonce: HolderInvocationNonce,
    services: TestServices<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    evaluate_action(
        compact_token,
        request,
        EnforcementAction::DocumentWrite,
        signature,
        request_nonce,
        &services,
    )
}

fn evaluate_action<'request>(
    compact_token: &str,
    request: &'request EnforcementRequest,
    action: EnforcementAction,
    signature: &SignerSignature,
    request_nonce: HolderInvocationNonce,
    services: &TestServices<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    let key_resolver = services.key_resolver;
    let proof_store = services.proof_store;
    let revocations = services.revocations;
    let caveats = services.caveats;
    let replay = services.replay;
    let verification_time = services.verification_time;
    let verification_limits = services.verification_limits;
    let holder_limits = services.holder_limits;
    let policy = default_policy().expect("embedded policy must parse");
    let policy_ref = default_authority_policy_ref(request.contract());
    let raw_request =
        UcanEnforcementRequest::new(request, compact_token, action, request_nonce, signature);
    let boundaries = UcanEnforcementBoundaries {
        verification_time,
        key_resolver,
        proof_store,
        revocations,
        caveat_policies: caveats,
        replay,
        verification_limits,
        holder_limits,
    };
    verify_ucan_enforcement_request(&policy, &policy_ref, &raw_request, &boundaries)
}

fn assert_denied(
    result: Result<LiveUcanEnforcement<'_>, UcanEnforcementFailure>,
    expected: UcanEnforcementReason,
) -> UcanEnforcementReceipt {
    let error = result.expect_err("negative adapter case must deny");
    assert_eq!(error.reason(), expected);
    assert!(!error.receipt().is_allowed());
    error.receipt().clone()
}

#[test]
fn direct_holder_invocation_returns_live_exact_request_authority() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    )
    .expect("valid direct holder invocation must pass");

    assert_eq!(
        std::ptr::from_ref(live.request()),
        std::ptr::from_ref(&request)
    );
    assert!(live.receipt().is_allowed());
    assert_eq!(live.holder_authorization().resource(), DOCUMENT_RESOURCE);
    assert_eq!(replay.calls.get(), FIRST_CALL);
    assert!(replay.committed.get());
}

#[test]
fn delegated_holder_invocation_returns_live_authority() {
    let fixture = InvocationFixture::delegated();
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    )
    .expect("valid delegated holder invocation must pass");

    assert!(live.receipt().is_allowed());
    assert_eq!(
        live.holder_authorization().ability(),
        DOCUMENT_WRITE_ABILITY
    );
    assert_eq!(replay.calls.get(), FIRST_CALL);
}

#[test]
fn caveat_observations_repeat_before_one_final_replay_commit() {
    let fixture = InvocationFixture::direct(caveated_document_capability());
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let caveats = CountingCaveatSet::new(true);
    let replay =
        RecordingReplay::after_caveats(&caveats.policy.calls, EXPECTED_CAVEAT_OBSERVATIONS);

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &caveats, &replay),
    )
    .expect("admitted caveat must pass both observations");

    assert!(live.receipt().is_allowed());
    assert_eq!(caveats.policy.calls.get(), EXPECTED_CAVEAT_OBSERVATIONS);
    assert_eq!(replay.calls.get(), FIRST_CALL);
    assert!(replay.committed.get());
}

#[test]
fn basalt_policy_denial_does_not_burn_nonce() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(PRIVATE_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::PolicyDenied,
    );

    assert_eq!(replay.calls.get(), 0);
    assert!(!replay.committed.get());
}

#[test]
fn final_verification_is_authoritative_and_drops_prospective_authority() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let resolver = FailsAfterResolver {
        inner: &fixture.keys,
        allowed_calls: EXPECTED_PREFLIGHT_RESOLVER_CALLS,
        calls: Cell::new(0),
    };
    let services = TestServices {
        key_resolver: &resolver,
        proof_store: &fixture.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    let receipt = assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services,
        ),
        UcanEnforcementReason::BackendUnavailable,
    );

    assert!(!receipt.is_allowed());
    assert_eq!(replay.calls.get(), 0);
    assert!(resolver.calls.get() > EXPECTED_PREFLIGHT_RESOLVER_CALLS);
}

#[test]
fn duplicate_nonce_fails_only_at_final_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let first = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    );
    assert!(first.is_ok());
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ReplayDenied,
    );

    assert_eq!(replay.calls.get(), EXPECTED_CAVEAT_OBSERVATIONS);
}

#[test]
fn caller_supplied_capability_facts_are_rejected_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE).with_capability(CapabilityGrant::new(
        DOCUMENT_RESOURCE,
        DOCUMENT_WRITE_ABILITY,
    ));
    let replay = RecordingReplay::new();
    let valid_request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &valid_request, nonce());

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::UnverifiedAuthorityFields,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn malformed_and_oversized_credentials_fail_closed() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let malformed = "not.enough";
    let oversized = "x".repeat(OVERSIZED_TOKEN_BYTES);
    let mut oversized_services = services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay);
    oversized_services.verification_limits = VerificationLimits::new(
        VerificationLimits::default().max_proof_depth(),
        VerificationLimits::default().max_proof_count(),
        OVERSIZED_TOKEN_LIMIT,
    );

    let malformed_receipt = assert_denied(
        evaluate(
            malformed,
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::MalformedCredential,
    );
    let oversized_receipt = assert_denied(
        evaluate(
            oversized.as_str(),
            &request,
            &signature,
            nonce(),
            oversized_services,
        ),
        UcanEnforcementReason::OversizedCredential,
    );

    assert!(malformed_receipt.credential_blake3.is_some());
    assert!(oversized_receipt.credential_blake3.is_none());
    assert_eq!(
        oversized_receipt.credential_bytes,
        u64::try_from(OVERSIZED_TOKEN_BYTES).expect("test token length must fit u64")
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn token_signature_and_holder_signature_failures_are_distinct() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let substituted_nonce_signature = sign_request(&fixture, &request, other_nonce());
    let replay = RecordingReplay::new();
    let wrong_issuer = signer(WRONG_ISSUER_SEED_BYTE);
    let wrong_key_resolver = WrongIssuerKeyResolver {
        public_key: wrong_issuer
            .public_key()
            .expect("wrong issuer public key must derive"),
    };
    let wrong_key_services = TestServices {
        key_resolver: &wrong_key_resolver,
        proof_store: &fixture.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            wrong_key_services,
        ),
        UcanEnforcementReason::SignatureDenied,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &substituted_nonce_signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn typed_action_and_profile_substitutions_fail_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let valid_signature = sign_request(&fixture, &request, nonce());
    let exact_operation = bind_enforcement_action(EnforcementAction::DocumentWrite, &request)
        .expect("test request must bind");
    let substituted_operation = OperationBinding::new(
        OperationProfileId::from_bytes([UNKNOWN_PROFILE_BYTE; HOLDER_INVOCATION_DIGEST_BYTES]),
        exact_operation.request_binding(),
    );
    let substituted_profile_signature =
        sign_operation(&fixture, &request, nonce(), substituted_operation);
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate_action(
            fixture.token.as_str(),
            &request,
            EnforcementAction::DocumentRead,
            &valid_signature,
            nonce(),
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::OperationMismatch,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &substituted_profile_signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn token_and_request_substitutions_invalidate_holder_signature() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let substituted_token_fixture =
        InvocationFixture::direct(document_capability(DOCUMENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let substituted_request = enforcement_request(SUBSTITUTED_REQUEST_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            substituted_token_fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(
                &substituted_token_fixture,
                &NoRevocations,
                &DenyUnknownCaveats,
                &replay,
            ),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &substituted_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn canonical_field_size_failure_has_stable_limit_reason() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let mut bounded_services = services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay);
    bounded_services.holder_limits = HolderInvocationLimits::try_new(
        HolderInvocationLimits::default().max_audience_bytes(),
        SHORT_CANONICAL_RESOURCE_LIMIT,
        HolderInvocationLimits::default().max_ability_bytes(),
    )
    .expect("short test canonical limit must validate");

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            bounded_services,
        ),
        UcanEnforcementReason::LimitExceeded,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn proof_and_key_failures_are_typed() {
    let delegated = InvocationFixture::delegated();
    let delegated_request = enforcement_request(DOCUMENT_RESOURCE);
    let delegated_signature = sign_request(&delegated, &delegated_request, nonce());
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let direct_request = enforcement_request(DOCUMENT_RESOURCE);
    let direct_signature = sign_request(&direct, &direct_request, nonce());
    let replay = RecordingReplay::new();
    let empty_proofs = ProofCollection::empty();
    let holder_only_keys = KeyResolutionContext::new(vec![
        direct
            .holder
            .verification_key()
            .expect("holder key must derive"),
    ]);

    let proof_services = TestServices {
        key_resolver: &delegated.keys,
        proof_store: &empty_proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };
    assert_denied(
        evaluate(
            delegated.token.as_str(),
            &delegated_request,
            &delegated_signature,
            nonce(),
            proof_services,
        ),
        UcanEnforcementReason::ProofDenied,
    );

    let key_services = TestServices {
        key_resolver: &holder_only_keys,
        proof_store: &direct.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };
    assert_denied(
        evaluate(
            direct.token.as_str(),
            &direct_request,
            &direct_signature,
            nonce(),
            key_services,
        ),
        UcanEnforcementReason::KeyResolutionDenied,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn time_revocation_caveat_and_scope_failures_are_typed() {
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let direct_signature = sign_request(&direct, &request, nonce());
    let expired = InvocationFixture::direct_with_expiration(
        document_capability(PARENT_RESOURCE),
        TOKEN_EXPIRES_SECONDS,
    );
    let expired_signature = sign_request(&expired, &request, nonce());
    let caveated = InvocationFixture::direct(caveated_document_capability());
    let caveated_signature = sign_request(&caveated, &request, nonce());
    let scoped = InvocationFixture::direct(document_capability(OTHER_RESOURCE));
    let scoped_signature = sign_request(&scoped, &request, nonce());
    let replay = RecordingReplay::new();
    let revoked = RevocationList::from_references(vec![proof_reference(&direct.token)]);
    let mut expired_services = services(&expired, &NoRevocations, &DenyUnknownCaveats, &replay);
    expired_services.verification_time =
        VerificationTime::from_unix_seconds(EXPIRED_VERIFY_AT_SECONDS);

    assert_denied(
        evaluate(
            expired.token.as_str(),
            &request,
            &expired_signature,
            nonce(),
            expired_services,
        ),
        UcanEnforcementReason::TimeDenied,
    );
    assert_denied(
        evaluate(
            direct.token.as_str(),
            &request,
            &direct_signature,
            nonce(),
            services(&direct, &revoked, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::Revoked,
    );
    assert_denied(
        evaluate(
            caveated.token.as_str(),
            &request,
            &caveated_signature,
            nonce(),
            services(&caveated, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::CaveatDenied,
    );
    assert_denied(
        evaluate(
            scoped.token.as_str(),
            &request,
            &scoped_signature,
            nonce(),
            services(&scoped, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ScopeDenied,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn operation_and_request_limits_fail_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let valid_request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &valid_request, nonce());
    let wrong_action_request =
        EnforcementRequest::new("document-write", DOCUMENT_RESOURCE, DOCUMENT_READ_ABILITY);
    let oversized_resource = "r".repeat(OVERSIZED_REQUEST_BYTES);
    let oversized_request = enforcement_request(oversized_resource.as_str());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &wrong_action_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::OperationMismatch,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &oversized_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::LimitExceeded,
    );

    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn identical_evaluations_have_identical_redacted_receipts() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let first_replay = RecordingReplay::new();
    let second_replay = RecordingReplay::new();

    let first = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &first_replay),
    )
    .expect("first evaluation must pass");
    let second = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(
            &fixture,
            &NoRevocations,
            &DenyUnknownCaveats,
            &second_replay,
        ),
    )
    .expect("second evaluation must pass");

    assert_eq!(first.receipt(), second.receipt());
    assert_eq!(first_replay.calls.get(), FIRST_CALL);
    assert_eq!(second_replay.calls.get(), FIRST_CALL);
}

#[test]
fn compact_bearer_fixtures_keep_zeroizing_ownership() {
    let source = include_str!("ucan_enforcement.rs");
    let docs = include_str!("../docs/ucan-enforcement.md");
    let forbidden = [
        ["token.as_str().as_bytes()", ".to_vec()"].concat(),
        ["token.as_str()", ".to_owned()"].concat(),
        ["token.as_str()", ".to_string()"].concat(),
        ["serde_json::to_string(&fixture.", "token"].concat(),
        [".arg(fixture.", "token.as_str())"].concat(),
        ["println!(fixture.", "token"].concat(),
        ["eprintln!(fixture.", "token"].concat(),
        ["dbg!(&fixture.", "token"].concat(),
        ["tampered_", "token"].concat(),
    ];

    for pattern in forbidden {
        assert!(
            !source.contains(pattern.as_str()),
            "test source copies or exposes compact bearer text: {pattern}"
        );
    }
    assert!(docs.contains("Pass only borrowed `CompactToken::as_str()` views"));
    assert!(docs.contains("Do not copy a compact bearer into `String` or `Vec` values"));
}

#[test]
fn persisted_receipt_cannot_recreate_live_authority() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    )
    .expect("valid invocation must pass");
    let persisted = serde_json::to_string(live.receipt()).expect("receipt must serialize");
    let restored: UcanEnforcementReceipt =
        serde_json::from_str(&persisted).expect("receipt must deserialize as evidence");
    let source = include_str!("../src/ucan_enforcement.rs");
    let struct_start = source
        .find("pub struct LiveUcanEnforcement")
        .expect("live type source must exist");
    let struct_end = source[struct_start..]
        .find("impl fmt::Debug for LiveUcanEnforcement")
        .map(|offset| struct_start + offset)
        .expect("live debug implementation must follow its declaration");
    let struct_block = &source[struct_start..struct_end];
    let failure_start = source
        .find("pub struct UcanEnforcementFailure")
        .expect("failure type source must exist");
    let failure_end = source[failure_start..]
        .find("impl UcanEnforcementFailure")
        .map(|offset| failure_start + offset)
        .expect("failure implementation must follow its declaration");
    let failure_block = &source[failure_start..failure_end];
    let live_impl = source
        .split("impl<'request> LiveUcanEnforcement")
        .nth(1)
        .expect("live implementation must exist")
        .split("#[derive(Clone, Copy)]")
        .next()
        .expect("live implementation must end before receipt context");

    assert_eq!(&restored, live.receipt());
    assert!(!persisted.contains("did:key"));
    assert!(!persisted.contains(DOCUMENT_WRITE_ABILITY));
    assert!(
        !struct_block
            .lines()
            .skip(1)
            .any(|line| line.trim_start().starts_with("pub "))
    );
    assert!(!failure_block.contains("AuthorityDecision"));
    assert!(!failure_block.contains("LiveUcanEnforcement"));
    assert!(!live_impl.contains("pub fn new"));
    assert!(!live_impl.contains("authority_decision"));
    assert!(!source.contains("impl Clone for LiveUcanEnforcement"));
    assert!(!source.contains("impl serde::Serialize for LiveUcanEnforcement"));
    assert!(!source.contains("impl serde::Deserialize for LiveUcanEnforcement"));
    assert!(!source.contains("#[derive(Clone)]\npub struct LiveUcanEnforcement"));
    assert!(!source.contains("#[derive(Serialize)]\npub struct LiveUcanEnforcement"));
    assert!(!source.contains("#[derive(Deserialize)]\npub struct LiveUcanEnforcement"));
}

#[test]
fn successful_replay_branch_only_moves_prebuilt_values() {
    let source = include_str!("../src/ucan_enforcement.rs");
    let success_branch = source
        .split("match verify_holder_signed_invocation")
        .nth(1)
        .expect("final holder verification call must exist")
        .split("Err(error)")
        .next()
        .expect("success branch must precede failure branch");

    for forbidden in [
        "build_receipt",
        "evaluate_authority",
        "adapt_enforcement_request",
        "blake3::",
        "format!",
        "assert!",
        "to_owned",
        "clone()",
    ] {
        assert!(
            !success_branch.contains(forbidden),
            "post-replay success branch contains forbidden work: {forbidden}"
        );
    }
    assert!(success_branch.contains("Ok(LiveUcanEnforcement"));
    assert!(success_branch.contains("_authority_decision: prospective.decision"));
    assert!(success_branch.contains("receipt: prospective.receipt"));
}
