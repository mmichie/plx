{
  description = "Powerline-styled path, git status, and tmux title segments";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      buildFor =
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          craneLib = crane.mkLib pkgs;
        in
        craneLib.buildPackage {
          src =
            let
              binFilter = path: _type: builtins.match ".*\\.bin$" path != null;
            in
            pkgs.lib.cleanSourceWith {
              src = ./.;
              filter =
                path: type:
                (binFilter path type) || (craneLib.filterCargoSources path type);
            };
          strictDeps = true;
          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.cmake
          ];
          buildInputs =
            [
              pkgs.openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.apple-sdk_15
              pkgs.libiconv
            ];
        };
    in
    {
      packages = forAllSystems (system: {
        default = buildFor system;
        plx = buildFor system;
      });

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);
    };
}
