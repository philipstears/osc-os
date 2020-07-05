let
  mozilla =
    builtins.fetchGit {
      name = "nixpkgs-mozilla";
      url = https://github.com/mozilla/nixpkgs-mozilla/;
    };

  nixPackages =
    import <nixpkgs> {
      overlays = [
        (import mozilla)
      ];
    };

  rustBase =
    (nixPackages.rustChannelOf { date = "2020-07-04"; channel = "nightly"; }).rust;

  rust =
    (rustBase.override {
      extensions = [
        "rust-src"
        "rls-preview"
        "rust-analysis"
        "rustfmt-preview"
        "clippy-preview"
      ];
    });

in

  with nixPackages;

  mkShell {
    buildInputs = with pkgs; [
      rust
      rustracer

      # Documentation build
      asciidoctor

      # QEMU
      qemu

      # OVMF firmware download
      python38Packages.rpm

      # Image building
      mtools
      parted
    ];

    RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/src";
    RACER_PATH = "${rustracer}/bin/racer";
  }
