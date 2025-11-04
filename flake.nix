{
  description = "Clarity - A Rust project with CUDA support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  # Configure binary cache for CUDA packages
  nixConfig = {
    extra-substituters = [ "https://cuda-maintainers.cachix.org" ];
    extra-trusted-public-keys = [ "cuda-maintainers.cachix.org-1:0dq3bujKpuEPMCX6U4WylrUDZ9JyUG0VpVZa7CNfq5E=" ];
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
            cudaSupport = true;
          };
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # CUDA packages
        inherit (pkgs.cudaPackages) cudatoolkit cuda_cccl cuda_cudart cuda_nvcc;
        inherit (pkgs.linuxPackages) nvidia_x11;

        # Library dependencies for Candle
        lib-packages = with pkgs; [
          stdenv.cc.cc.lib
          zlib
          cuda_cudart
          cuda_cccl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy

            # CUDA development tools
            cuda_nvcc
            cudatoolkit
            cuda_cccl
            cuda_cudart

            # System tools
            pkg-config
            openssl
          ];

          nativeBuildInputs = [
            cudatoolkit
            cuda_nvcc
          ];

          # CUDA environment variables
          shellHook = ''
            export CUDA_PATH=${cudatoolkit}
            export CUDA_ROOT=${cudatoolkit}
            export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath lib-packages}

            # Add NVIDIA drivers on NixOS
            if [ -d /etc/nixos ]; then
              export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${nvidia_x11}/lib
              export EXTRA_LDFLAGS="-L/lib -L${nvidia_x11}/lib"
            fi

            export EXTRA_CCFLAGS="-I/usr/include"

            # Verify CUDA setup
            echo "CUDA toolkit: ${cudatoolkit}"
            echo "nvcc version:"
            nvcc --version 2>/dev/null || echo "nvcc not found"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "clarity";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    );
}
