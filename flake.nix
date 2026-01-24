{
  description = "ndown - A Makepad-based markdown block editor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
      "https://cache.nixos.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
    ];
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        # Overlays and package set
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        # Rust toolchain configuration
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = ["rust-src"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (
                file:
                  pkgs.lib.any file.hasExt [
                    "svg"
                  ]
              )
              unfilteredRoot)
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        # Dependencies for building the application
        buildInputs = with pkgs; [
          wayland
          vulkan-loader
          mesa
          libglvnd
          xorg.libxcb
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          libxkbcommon
          fontconfig
          openssl
          freetype
          alsa-lib
          libpulseaudio
          nerd-fonts.ubuntu
          nerd-fonts.ubuntu-mono
          nerd-fonts.ubuntu-sans
        ];

        # Dependencies needed only at runtime
        runtimeDependencies = with pkgs; [
          wayland
          vulkan-loader
          mesa
          libxkbcommon
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
          clang
          mold
        ];

        envVars = {
          RUST_BACKTRACE = "full";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold -C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath [pkgs.vulkan-loader pkgs.wayland]}";
          NIX_LDFLAGS = "-rpath ${pkgs.lib.makeLibraryPath [pkgs.vulkan-loader pkgs.wayland]}";
        };

        # Build artifacts
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs nativeBuildInputs;
          env = envVars;
        };

        # Application package definition
        ndown = craneLib.buildPackage {
          inherit src cargoArtifacts buildInputs nativeBuildInputs runtimeDependencies;
          env = envVars;
          pname = "ndown";
          version = "0.1.0";

          # Prevent nix from removing "unused" wayland/vulkan rpaths
          dontPatchELF = true;

          postFixup = ''
            # Copy assets to the output
            mkdir -p $out/share/ndown
            cp -r ${src}/assets $out/share/ndown/

            wrapProgram $out/bin/ndown \
              --prefix LD_LIBRARY_PATH : /run/opengl-driver/lib:${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)} \
              --set NDOWN_ASSETS_DIR $out/share/ndown/assets \
              --set __EGL_VENDOR_LIBRARY_FILENAMES /run/opengl-driver/share/glvnd/egl_vendor.d/50_mesa.json:/run/opengl-driver/share/glvnd/egl_vendor.d/10_nvidia.json \
              --run 'export VK_ICD_FILENAMES=$(find /run/opengl-driver/share/vulkan/icd.d -name "*_icd.*.json" 2>/dev/null | tr "\n" ":" | sed "s/:$//")' \
              --run 'if lspci 2>/dev/null | grep -qi nvidia; then export __GL_THREADED_OPTIMIZATIONS=1 __GL_YIELD=USLEEP NDOWN_GPU=nvidia; elif lspci 2>/dev/null | grep -qi amd; then export MESA_GLTHREAD=true RADV_PERFTEST=gpl NDOWN_GPU=amd; fi'
          '';
        };

        # Development shell tools
        devTools = with pkgs; [
          rust-analyzer
          rustToolchain
          cargo-watch
          cargo-edit
          cargo-audit
          cargo-machete
          cargo-bloat
          cargo-flamegraph
          bacon
          libnotify
        ];
      in {
        packages = {
          default = ndown;
          inherit ndown;
        };

        checks = {
          inherit ndown;

          ndown-clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts buildInputs nativeBuildInputs;
            env = envVars;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          ndown-fmt = craneLib.cargoFmt {inherit src;};
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ndown];
          nativeBuildInputs = devTools;
          env = envVars;

          # Vulkan libs first, then system GL
          LD_LIBRARY_PATH = "/run/opengl-driver/lib:${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}";
          __EGL_VENDOR_LIBRARY_FILENAMES = "/run/opengl-driver/share/glvnd/egl_vendor.d/50_mesa.json:/run/opengl-driver/share/glvnd/egl_vendor.d/10_nvidia.json";
          FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = buildInputs;};

          shellHook = ''
            echo "[🦀 Rust $(rustc --version)] - Ready to develop ndown!"

            # Auto-detect available Vulkan ICDs
            export VK_ICD_FILENAMES=$(find /run/opengl-driver/share/vulkan/icd.d -name "*_icd.*.json" 2>/dev/null | tr '\n' ':' | sed 's/:$//')

            # GPU-specific optimizations
            if lspci 2>/dev/null | grep -qi nvidia; then
              export __GL_THREADED_OPTIMIZATIONS=1
              export __GL_YIELD=USLEEP
              export NDOWN_GPU=nvidia
              echo "GPU: NVIDIA detected - threaded optimizations enabled"
            elif lspci 2>/dev/null | grep -qi amd; then
              export MESA_GLTHREAD=true
              export RADV_PERFTEST=gpl
              export NDOWN_GPU=amd
              echo "GPU: AMD detected - RADV optimizations enabled"
            else
              echo "GPU: Auto-detected (AMD/NVIDIA/Intel)"
            fi

            echo "Vulkan ICD: $VK_ICD_FILENAMES"
            echo "Available Vulkan devices:"
            vulkaninfo --summary 2>/dev/null | grep -A 2 "GPU" || echo "  Run 'vulkaninfo' for details"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
