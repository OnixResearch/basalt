# Changelog

## Unreleased

### Changed

- The OIDC exchange is one-time per OIDC token (issue #7). The replay key now
  covers only the token issuer and `jti`, under the new domain tag
  `basalt.oidc.replay.v2`. A second exchange of the same token is a duplicate,
  even for another operation, resource or holder. Consumers mint a fresh OIDC
  token for each exchange.
- The shell reserves the replay key until
  `max(exp, iat + max_token_age_seconds) + clock_skew_seconds`, and no longer
  only until the issued UCAN expires. `IssuancePlan` gains
  `replay_expires_at_seconds`, which is the value passed to `ReplayPort::commit`.

### Upgrade notes

- Reservations made under the old `basalt.oidc.replay.v1` keys do not carry
  over. A token exchanged just before the upgrade can be exchanged once more
  under the new key. The token's own lifetime bounds that window: verification
  rejects it after `max(exp, iat + max_token_age_seconds) + clock_skew_seconds`.
  Under the shipped policy that is 630 seconds after `iat` for a token that
  expires within its 600-second age limit.
