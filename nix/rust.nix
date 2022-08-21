{
	pkgs,
	channel,
	version
}: let
	toolchain = pkgs.rust-bin.${channel}.${version}.default.override {
		extensions = [ "rust-src" "rustfmt" "rust-analyzer" ];
		targets = [
			(pkgs.rust.toRustTarget pkgs.stdenv.buildPlatform)
			(pkgs.rust.toRustTarget pkgs.stdenv.hostPlatform)
			"wasm32-unknown-unknown"
		];
	};

	project = pkgs.rustBuilder.makePackageSet {
		rustChannel = toolchain;
		packageFun = import ../Cargo.nix;
		target = null;
	};

	platform = pkgs.makeRustPlatform {
		cargo = toolchain;
		rustc = toolchain;
	};
in {
	inherit toolchain project platform;
}
