{ pkgs, ... }:

{
  languages.rust.enable = true;

  packages = with pkgs; [
    alsa-utils
    lame
  ];

  env = {
    RUST_LOG = "info";
  };
}
