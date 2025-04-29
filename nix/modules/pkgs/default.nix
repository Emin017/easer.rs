{
  pkgs,
  ...
}:
let
  self = {
    packages = {
      easer = pkgs.callPackages ./easer.nix { };
      default = self.packages.easer;
    };
  };
in
self
