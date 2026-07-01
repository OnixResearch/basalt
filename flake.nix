{
  description = "Basalt UCAN contract boundary toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    ucan = {
      url = "github:OnixResearch/ucan/ad61b53e89fa45f9bf7d313ce14c45de645bf53d";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ucan,
    }:
    {
      lib.ucanSource = ucan;
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
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
            }
            ''
              export HOME="$TMPDIR/home"
              export CARGO_HOME="$TMPDIR/cargo-home"
              export RELEASE_VERIFY_RECEIPT_DIR="$out"
              export RELEASE_VERIFY_POLICY_FRESH=1
              export RELEASE_VERIFY_CARGO_STEPS=0
              mkdir -p "$HOME" "$CARGO_HOME" "$out"
              rustc ${./tools/release-verify.rs} -o "$TMPDIR/basalt-release-verify"
              cd ${./.}
              "$TMPDIR/basalt-release-verify"
            '';
      }
    );
}
