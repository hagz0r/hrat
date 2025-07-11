{
  description = "Development environment for the hrat C&C web host";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      pythonEnv = pkgs.python3.withPackages (ps: [
        ps.fastapi
        ps.uvicorn
        ps.jinja2
        ps.python-multipart
      ]);

    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          pythonEnv


          cargo
          rustc
          pkg-config

          xorg.libX11
          xorg.libxcb
        ];

        shellHook = ''
          echo ""
          echo "--- hrat C&C Host Environment Activated ---"
          echo ""
          echo "Python with FastAPI is now available."
          echo "Rust toolchain and build dependencies are also available."
          echo ""
          echo "To run the web server:"
          echo "  uvicorn main:app --reload"
          echo ""
          echo "To build the client manually (for testing):"
          echo "  cd ../client && cargo build"
          echo ""
        '';
      };
    };
}
