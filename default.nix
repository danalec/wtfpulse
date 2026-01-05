{ pkgs ? import <nixpkgs> {} }:

let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;
  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl ];

  meta = with pkgs.lib; {
    description = manifest.description;
    homepage = "https://github.com/danalec/wtfpulse";
    license = with licenses; [ mit apache2 ];
    maintainers = [ ];
  };
}
