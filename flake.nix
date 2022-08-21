{
	description = "tarpc-wasm";

	inputs = {
		flake-compat = { url = "github:edolstra/flake-compat"; flake = false; };
		nixpkgs      = { url = "github:nixos/nixpkgs/nixos-unstable"; };
		flake-utils  = { url = "github:numtide/flake-utils"; inputs.nixpkgs.follows = "nixpkgs"; };
		cargo2nix    = { url = "github:cargo2nix/cargo2nix"; inputs.nixpkgs.follows = "nixpkgs"; };
		rust-overlay = { url = "github:oxalica/rust-overlay"; inputs = { nixpkgs.follows = "nixpkgs"; flake-utils.follows = "flake-utils"; }; };
	};

	outputs = { self, nixpkgs, cargo2nix, flake-utils, rust-overlay, ... }:
		flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [
						cargo2nix.overlays.default
						rust-overlay.overlays.default
					];
				};

				node_pkg = pkgs.nodejs-18_x.override { enableNpm = false; };

				rust = import ./nix/rust.nix {
					inherit pkgs;
					channel = "nightly";
					version = "2022-08-16";
				};
			in rec {
				devShell = pkgs.mkShell rec {
					nativeBuildInputs = with pkgs; [
						binaryen
						cargo-expand
						rust.toolchain
						wasm-bindgen-cli
						wasm-pack

						node_pkg
						(yarn.override { nodejs = node_pkg; }) # by default yarn uses the latest version of nodejs, so we override it to the correct version here
					];
				};
			}
		);
}
