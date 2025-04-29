{
  pkgs,
  ...
}:
let
  self = {
    packages = {
      easer = pkgs.callPackage ./easer.nix { };
      default = self.packages.easer;
    };
  };
in
self
