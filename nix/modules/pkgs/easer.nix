{
  pkgs,
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  rustPackages,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "easer";
  version = "0.1.0";

  cargoLock = {
    lockFile = ./../../Cargo.lock;
  };
  src =
    with lib.fileset;
    toSource {
      root = ./../../..;
      fileset = unions [
        ./../../../src
        ./../../../Cargo.lock
        ./../../../Cargo.toml
      ];
    };
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
  ];
}
