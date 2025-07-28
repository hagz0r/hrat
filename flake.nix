{
  description = "Dev shell for hrat host + client (PipeWire / SPA OK)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      includePkgs = [
        pkgs.glibc.dev
        pkgs.llvmPackages.clang.cc
        pkgs.pipewire.dev
        pkgs.linuxHeaders
        pkgs.libv4l.dev
        pkgs.libxkbcommon
      ];

      baseInc = pkgs.lib.makeSearchPath "include" includePkgs;
      spaInc = "${pkgs.pipewire.dev}/include/spa-0.2";
      devInc = "${baseInc}:${spaInc}";

      bindgenFlags = pkgs.lib.concatStringsSep " " (
        map (p: "-I${p}/include") includePkgs ++ [ "-I${spaInc}" ]
      );

      devLib = pkgs.lib.makeLibraryPath [
        # Wayland runtime deps для winit/eframe
        pkgs.wayland # libwayland-client.so.*
        pkgs.libxkbcommon # libxkbcommon.so.*

        # Часто полезные графические либы (X11/Xcursor/Xrandr, GL, и т.п.)
        pkgs.xorg.libX11
        pkgs.xorg.libXi
        pkgs.xorg.libXcursor
        pkgs.xorg.libXrandr
        pkgs.libGL
        pkgs.mesa
        pkgs.vulkan-loader

        # То, что уже было/может понадобиться
        pkgs.pipewire
        pkgs.libv4l
        pkgs.alsa-lib
        pkgs.fontconfig
        pkgs.freetype
      ];

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
          cargo
          rustc
          gcc
          llvmPackages.clang
          llvmPackages.libclang
          pkg-config
          glibc.dev
          linuxHeaders
          libv4l.dev
          pipewire
          alsa-lib
          udev
          xorg.libX11
          xorg.libxcb
          dbus
          pythonEnv
          wayland
          wayland-protocols
          wayland-utils
        ];

        C_INCLUDE_PATH = devInc;
        CPLUS_INCLUDE_PATH = devInc;
        LIBRARY_PATH = devLib;
        LD_LIBRARY_PATH = devLib;

        shellHook = ''
          								# bindgen
          								export BINDGEN_EXTRA_CLANG_ARGS="${bindgenFlags}"
          								export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"

          								echo
          								echo "--- hrat dev shell ready ---"
          								echo
          	echo "cd client/ && RAT_HOST_IP=127.0.0.1 RAT_HOST_PORT=8080 cargo run --release"
          								echo "cd host/ && uvicorn main:app --reload"
          								echo


          								zeditor host
          	zeditor client
          						'';
      };
    };
}
