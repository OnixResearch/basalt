{
  description = "Basalt UCAN contract boundary toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        validAuthorityFixtures = [
          "positive/direct-match"
          "positive/delegated-match"
          "negative/missing-grant"
          "negative/wrong-resource"
          "negative/wrong-ability"
          "negative/stale-verification-ref"
          "negative/caveat-denied"
          "negative/replay-denied"
          "negative/missing-policy-ref"
          "negative/malformed-request-ref"
          "negative/authority-overclaim"
          "negative/missing-verification-receipt"
          "negative/verification-request-mismatch"
        ];
        malformedAuthorityFixtures = [
          "weakened-non-claims"
          "invalid-disposition"
          "incompatible-expected"
        ];
        validUcanSemanticsFixtures = [
          "positive/allow"
          "positive/capability-deny"
          "negative/over-delegation"
          "negative/wildcard-request"
          "negative/missing-caveat-policy"
          "negative/stale-proof"
          "negative/replay-denied"
        ];
        malformedUcanSemanticsFixtures = [
          "missing-operation"
          "weakened-non-claims"
          "incompatible-expected-result"
          "stale-catalog"
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.b3sum
            pkgs.cargo
            pkgs.cargo-nextest
            pkgs.clippy
            pkgs.nickel
            pkgs.openssh
            pkgs.rustc
            pkgs.rustfmt
            pkgs.stdenv.cc
          ];
        };

        checks.vendored-ucan =
          pkgs.runCommand "basalt-vendored-ucan"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.rustc
                pkgs.stdenv.cc
              ];
            }
            ''
              rustc --edition=2024 --test ${./tools/sync-vendored-ucan.rs} -o vendored-ucan-tests
              ./vendored-ucan-tests
              rustc --edition=2024 ${./tools/sync-vendored-ucan.rs} -o vendored-ucan-check
              checker="$PWD/vendored-ucan-check"
              cd ${./.}
              "$checker" --check
              touch $out
            '';

        checks.policy-export =
          pkgs.runCommand "basalt-policy-export"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              nickel export ${./contracts/policy.ncl} > exported-policy.json
              cmp exported-policy.json ${./contracts/generated/policy.json}
              cp exported-policy.json $out
            '';

        checks.authority-core-fixtures =
          pkgs.runCommand "basalt-authority-core-fixtures"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              fixture_root=${./contracts/authority-core}/fixtures
              mkdir -p generated/positive generated/negative
              for fixture in ${builtins.concatStringsSep " " validAuthorityFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "generated/$fixture.json"
                cmp "generated/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " malformedAuthorityFixtures}; do
                if nickel export --format json "$fixture_root/malformed/$fixture.ncl" > /dev/null 2>"$fixture.err"; then
                  echo "malformed authority-core fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              cp -r generated "$out"
            '';

        checks.ucan-semantics-fixtures =
          pkgs.runCommand "basalt-ucan-semantics-fixtures"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              fixture_root=${./contracts/ucan-semantics}/fixtures
              mkdir -p generated/positive generated/negative
              for fixture in ${builtins.concatStringsSep " " validUcanSemanticsFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "generated/$fixture.json"
                cmp "generated/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " malformedUcanSemanticsFixtures}; do
                if nickel export --format json "$fixture_root/malformed/$fixture.ncl" > /dev/null 2>"$fixture.err"; then
                  echo "malformed UCAN semantics fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              cp -r generated "$out"
            '';

        checks.downstream-compat-config =
          pkgs.runCommand "basalt-downstream-compat-config"
            {
              nativeBuildInputs = [
                pkgs.rustc
                pkgs.stdenv.cc
              ];
            }
            ''
              rustc --edition=2024 --test ${./tools/downstream-compat.rs} -o downstream-compat-tests
              ./downstream-compat-tests
              rustc --edition=2024 ${./tools/downstream-compat.rs} -o downstream-compat
              ./downstream-compat --basalt-root ${./.} --list
              touch $out
            '';

        checks.release =
          pkgs.runCommandLocal "basalt-release-verify"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.cargo
                pkgs.cargo-nextest
                pkgs.clippy
                pkgs.git
                pkgs.nickel
                pkgs.openssh
                pkgs.rustc
                pkgs.rustfmt
                pkgs.stdenv.cc
              ];
              authorityCoreFixtures = self.checks.${system}.authority-core-fixtures;
              policyExport = self.checks.${system}.policy-export;
              ucanSemanticsFixtures = self.checks.${system}.ucan-semantics-fixtures;
            }
            ''
              export HOME="$TMPDIR/home"
              export CARGO_HOME="$TMPDIR/cargo-home"
              export RELEASE_VERIFY_RECEIPT_DIR="$out"
              export RELEASE_VERIFY_POLICY_FRESH=1
              export RELEASE_VERIFY_AUTHORITY_FRESH=1
              export RELEASE_VERIFY_SEMANTICS_FRESH=1
              export RELEASE_VERIFY_CARGO_STEPS=0
              mkdir -p "$HOME" "$CARGO_HOME" "$out" "$TMPDIR/release-source/tools" "$TMPDIR/release-source/src"
              cp ${./tools/release-verify.rs} "$TMPDIR/release-source/tools/release-verify.rs"
              cp ${./src/authority_identity.rs} "$TMPDIR/release-source/src/authority_identity.rs"
              cp ${./src/ucan_semantics_identity.rs} "$TMPDIR/release-source/src/ucan_semantics_identity.rs"
              rustc "$TMPDIR/release-source/tools/release-verify.rs" -o "$TMPDIR/basalt-release-verify"
              cd ${./.}
              "$TMPDIR/basalt-release-verify"
            '';
      }
    );
}
