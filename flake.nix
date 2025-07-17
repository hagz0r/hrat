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
        ps.websockets
      ]);

    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          pythonEnv

          cargo
          rustc
          pkg-config

          llvmPackages.libclang
          linuxHeaders
          
          # We are referencing glibc.dev in the shellHook, so it is good 
          # practice to make it an explicit input, even though it's
          # part of the standard environment.
          glibc.dev

          xorg.libX11
          xorg.libxcb
        ];

        shellHook = ''
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"

          # Add include paths for BOTH the Linux headers and the C Standard Library
          export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include"

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
