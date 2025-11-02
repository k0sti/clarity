{
  description = "Dioxus Assistant Application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            xdotool

            # GTK and WebKit dependencies for Dioxus desktop
            gtk3
            webkitgtk_4_1
            pkg-config
            glib
            cairo
            pango
            gdk-pixbuf
            atk
            libsoup_3

            # Wayland dependencies
            wayland
            wayland-protocols
            libxkbcommon
          ];

          shellHook = ''
            echo "Dioxus Assistant Development Environment"
            echo "Run: cargo run -p assistant"
          '';
        };
      }
    );
}
