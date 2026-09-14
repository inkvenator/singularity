{
  description = "glium/winit dev shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo
          rustc
        ];

        buildInputs = with pkgs; [
          libGL
          libX11
          libXcursor
          libXi
          libXrandr
          wayland
          libxkbcommon
        ];

        LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
          libGL
          libX11
          libXcursor
          libXi
          libXrandr
          wayland
          libxkbcommon
        ];
      };
    };
}