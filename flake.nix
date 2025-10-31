{
  description = "A tool to update my school logbook directly from the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs@{ flake-parts, crane, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
      ];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { pkgs, ...}:
        let
            craneLib = crane.mkLib pkgs;
        in {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = with pkgs; [
            openssl
            pkg-config
          ];
        };
        devshells.default = {
          env = [
            {
              name = "OPENSSL_DIR";
              value = "${pkgs.openssl.dev}";
            }
            {
              name = "OPENSSL_LIB_DIR";
              value = "${pkgs.openssl.out}/lib";
            }
            {
              name = "OPENSSL_NO_VENDOR";
              value = 1;
            }
          ];
          packages = with pkgs; [
            openssl
            pkg-config
          ];
        };
      };
    };
}
