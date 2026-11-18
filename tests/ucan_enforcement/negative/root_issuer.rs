use crate::support::*;

/// A resolvable key that is not an admitted root issues a proof-less grant.
fn forged_root_fixture() -> InvocationFixture {
    let forger = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
    let token = issue_token_with_signer(
        &forger,
        &audience,
        &CapabilitySet::new(vec![document_capability(PARENT_RESOURCE)])
            .expect("forged capability must validate"),
        &ProofReferences::empty(),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS)
            .expect("test time bounds must validate"),
    )
    .expect("forged token must issue");
    // Same resolver shape as every other fixture: holder keys must resolve
    // for the holder signature, so the forger's key resolves too.
    let keys = KeyResolutionContext::new(vec![
        signer(ISSUER_SEED_BYTE)
            .verification_key()
            .expect("issuer key must derive"),
        forger.verification_key().expect("forger key must derive"),
        holder.verification_key().expect("holder key must derive"),
    ]);
    InvocationFixture {
        token,
        holder,
        audience,
        keys,
        proofs: ProofCollection::empty(),
    }
}

#[test]
fn resolvable_non_root_key_cannot_mint_live_authority() {
    let fixture = forged_root_fixture();
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let receipt = assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::RootIssuerDenied,
    );

    assert_eq!(receipt.reason, UcanEnforcementReason::RootIssuerDenied);
    assert_eq!(replay.calls.get(), 0);
    assert!(!replay.committed.get());
}

#[test]
fn delegation_from_an_admitted_root_still_passes() {
    // Guard against over-correction: the delegated fixture's chain ends at the
    // admitted root even though its leaf issuer (the delegate) is not a root.
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
    .expect("delegation rooted at the admitted issuer must pass");

    assert!(live.receipt().is_allowed());
    assert_eq!(replay.calls.get(), FIRST_CALL);
}
