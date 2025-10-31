{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
      ];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      flake = {
      };
      perSystem = { config, pkgs, ...}: {
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
          packages = [
            pkgs.openssl
            pkgs.pkg-config
          ];
        };
      };
    };
}
