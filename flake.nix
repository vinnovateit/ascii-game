{
  description = "Project development tools for NixOS; application dependencies retain their own lockfiles";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/26f4107e47e3ad0ffd0db220469ba83b6f6d96e8";
  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [ rustup pkg-config python313 uv ];
        shellHook = ''
          export RUSTUP_HOME="''${XDG_CACHE_HOME:-$HOME/.cache}/vinnovateit/rustup"
          export CARGO_HOME="''${XDG_CACHE_HOME:-$HOME/.cache}/vinnovateit/cargo"
        '';
      };
    };
}
