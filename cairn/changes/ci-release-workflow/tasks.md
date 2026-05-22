# Tasks

- [ ] [serial] Add a checked-in CI workflow that runs the sandbox-safe flake release gate. r[crate_consumability.ci_release_workflow.default_flake_ci]
- [ ] [serial] Ensure the workflow does not require or expose repository secrets for the default public/sandbox-safe job. r[crate_consumability.ci_release_workflow.secret_safe_default]
- [ ] [serial] Document how maintainers run the full direct release verifier in CI environments with private UCAN dependency access. r[crate_consumability.ci_release_workflow.full_verifier_private_access]
- [ ] [serial] Preserve or document release-verifier receipt artifacts produced by CI. r[crate_consumability.ci_release_workflow.receipt_artifacts]
- [ ] [serial] Run workflow static review plus Cairn validation, `git diff --check`, and `nix flake check -L` before archive. r[crate_consumability.ci_release_workflow.default_flake_ci]
