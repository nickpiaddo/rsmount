{
  # Nixpkgs / NixOS version to use.
  inputs.nixpkgs.url = "nixpkgs/nixos-23.11";

  # Backward compatibility for people without flakes enabled.
  # https://github.com/edolstra/flake-compat
  inputs.flake-compat = {
    url = "github:edolstra/flake-compat";
    flake = false;
  };

  # Set of functions to make flake nix packages simpler to set up without
  # external dependencies.
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.naersk.url = "github:nix-community/naersk/master";

  outputs = { self, nixpkgs, flake-compat, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let

        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };

      in rec
      {
        # Development environment
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            git

            # Rust
            cargo
            rustc

            # Required by `bindgen`
            clang
            libclang.lib
            # `libblkid` source files
            util-linux.dev
          ];

          # Rust source path
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          # Required by `bindgen`
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          # Inspired by: "C header includes in NixOS"
          # https://discourse.nixos.org/t/c-header-includes-in-nixos/17410
          # Solve the error message when trying to compile libblkid-sys from inside test-microvm.
          # --- stderr
          # src/wrapper.h:1:10: fatal error: 'blkid/blkid.h' file not found
          C_INCLUDE_PATH="${pkgs.util-linux.dev}/include";
        };

      });
}
