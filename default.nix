{ pkgs ? import <nixpkgs> { } }:
let
  mf = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = mf.name;
    version = mf.version;
    src = pkgs.lib.cleanSource ./.;

    cargoLock.lockFile = ./Cargo.lock;
            
    CONFIG = "{}";  # not used, somewhere int eh futurr

    cargoHash = "sha256-SkFGStZShqocYwzyU7ylaQZ2+YRmHNCUqkCAvwFt1+c=";#nixpkgs.lib.fakeHash;
    #cargoSha256 = nixpkgs.lib.fakeSha256;
}
