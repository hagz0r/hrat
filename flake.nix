{
  description = "Dev shell and cross-compilation for hrat host + client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    nativeSystem = "x86_64-linux";
    crossSystem = "x86_64-w64-mingw32";

    pkgs = import nixpkgs {
      system = nativeSystem;
      overlays = [ rust-overlay.overlays.default ];
    };

    pkgsCross = import nixpkgs {
      system = nativeSystem;
      crossSystem = {
        config = crossSystem;
        openssl.static = true;
      };
    };

    rustToolchain = pkgs.rust-bin.stable.latest.default.override {
      targets = ["x86_64-pc-windows-gnu"];
    };

    pythonEnv = pkgs.python3.withPackages (ps: [
      ps.fastapi ps.uvicorn ps.jinja2 ps.python-multipart ps.websockets
    ]);
  in
  {
    # --- СРЕДА РАЗРАБОТКИ ДЛЯ LINUX ---
    devShells.${nativeSystem}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        rustToolchain pkg-config gcc llvmPackages.clang llvmPackages.libclang
        glibc.dev linuxHeaders libv4l.dev pipewire alsa-lib udev
        xorg.libX11 xorg.libxcb dbus
        pythonEnv
      ];
      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      shellHook = ''
        echo "--- hrat dev shell (Linux) ready ---"
      '';
    };

    # --- ПАКЕТЫ ДЛЯ СБОРКИ ---
    packages.${nativeSystem} = {
      # --- СБОРКА КЛИЕНТА ПОД WINDOWS ---
      hrat-client-windows =
        pkgsCross.rustPlatform.buildRustPackage {
          pname = "hrat-client";
          version = "0.1.0";

          # ИСПРАВЛЕНИЕ №1: Указываем весь проект как источник.
          # `self` — это специальная ссылка на директорию, где лежит flake.nix.
          src = self;

          # ИСПРАВЛЕНИЕ №2: Указываем сборщику, что нужно работать
          # в поддиректории `client` внутри исходников.
          sourceRoot = "source/client";

          # ИСПРАВЛЕНИЕ №3: Теперь путь к lock-файлу указывается
          # как и раньше, относительно корня проекта.
          cargoLock.lockFile = ./client/Cargo.lock;

          nativeBuildInputs = [
            rustToolchain
          ];

          buildInputs = [ ];
        };
    };
  };
}
