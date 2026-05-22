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
            pkgs.clippy
            pkgs.nickel
            pkgs.rustc
            pkgs.rustfmt
          ];
        };

        checks.policy-export = pkgs.runCommand "ucan-nickel-policy-export" {
          nativeBuildInputs = [ pkgs.nickel ];
        } ''
          nickel export ${./contracts/policy.ncl} > exported-policy.json
          cmp exported-policy.json ${./contracts/generated/policy.json}
          cp exported-policy.json $out
        '';
      });
}
