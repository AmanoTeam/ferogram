{
  pkgs,
  rustToolchain,
  ...
}:
pkgs.stdenv.mkDerivation rec {
  name = "ferogram-dev";

  # Compile time dependencies
  nativeBuildInputs = with pkgs; [
    # Rust
    rustToolchain
    rustPlatform.bindgenHook
  ];

  # Rust variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  # Compiler LD variables
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
    nativeBuildInputs
    ++ [
      pkgs.llvmPackages.llvm
    ]
  );
}
