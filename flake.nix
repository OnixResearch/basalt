{
  description = "UCAN + Nickel contract enforcement scaffold";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          packages = [
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

        checks.policy-export = pkgs.runCommand "ucan-nickel-policy-export" {
          nativeBuildInputs = [ pkgs.nickel ];
        } ''
          nickel export ${./contracts/policy.ncl} > exported-policy.json
          cmp exported-policy.json ${./contracts/generated/policy.json}
          cp exported-policy.json $out
        '';

        checks.release = pkgs.runCommandLocal "ucan-nickel-release-verify" {
          nativeBuildInputs = [
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
          policyExport = self.checks.${system}.policy-export;
        } ''
          export HOME="$TMPDIR/home"
          export CARGO_HOME="$TMPDIR/cargo-home"
          export RELEASE_VERIFY_RECEIPT_DIR="$out"
          export RELEASE_VERIFY_POLICY_FRESH=1
          export RELEASE_VERIFY_CARGO_STEPS=0
          mkdir -p "$HOME" "$CARGO_HOME" "$out"
          rustc ${./tools/release-verify.rs} -o "$TMPDIR/ucan-nickel-release-verify"
          cd ${./.}
          "$TMPDIR/ucan-nickel-release-verify"
        '';
      });
}
