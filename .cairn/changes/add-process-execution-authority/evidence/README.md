# Candidate verification — not release acceptance

Implementation source: `d2a3fe322b3e0c335e5737290d46c63bfaa7cb21`.
Parent: `89675cd4f585f837323c049e4a25f7b94c903038`.

The baseline passed 24 compact enforcement tests. The candidate passed 30 tests,
including six new signed execution-purpose controls. Strict all-target/all-feature
Clippy passed after a private-test-helper visibility correction. Proposal, design,
and task gates passed with the repository policy. No new suppression was added.

The complete `nix flake check -L` passed. This includes the opt-in policy freshness
check. The sandboxed release check defers Cargo-dependent acceptance by design.
It is not a substitute for the direct release verifier.

The direct verifier passed policy, source, license, and format stages, then failed
at library-test linking. Mold reported `failed to write to an output file. Disk full?`
and SIGBUS. A pinned coreutils check observed `/tmp` at 100% of its 61 GiB tmpfs,
with 144 KiB available. The home filesystem had 421 GiB available.

A development-only private mount namespace successfully mapped home-backed storage
to `/tmp`. The unchanged release verifier then failed to spawn `rustc` with
`Permission denied (os error 13)`. A shell in that namespace could run `rustc --version`.
The cause of that process-launch difference is unresolved. No release success is
inferred from the namespace probe. No host mount or unrelated file was changed.

The full release gate remains blocked. The candidate is not published or adopted
by Animus or Neural Stream. All lifecycle tasks remain open pending complete evidence.
Default policy bytes, existing operation fields, dependency pins, vendor snapshot,
Cargo lock, and Nix lock remain unchanged. No process, trainer, or promotion ran.

Next: provide a reviewed explicit scratch-directory option in the release runner,
or obtain adequate temporary storage, then rerun the complete unweakened verifier.
Do not bypass failed checks or treat the test keys as operator authority.

Logs and failed receipts are retained in the initiating Neural Stream checkout:
`.pi/drains/2026-09-07-execution-authority/`.
