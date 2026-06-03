{
  description = "Standalone terminal screen animations from Yazelix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPkgs = system: nixpkgs.legacyPackages.${system};
      yzsPackage =
        system: pkgs:
        let
          rustToolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
          ];
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
          source = pkgs.lib.cleanSourceWith {
            name = "yzs-source";
            src = ./.;
            filter =
              path: _type:
              let
                relativePath = pkgs.lib.removePrefix ((toString ./.) + "/") (toString path);
              in
              relativePath != "target"
              && !pkgs.lib.hasPrefix "target/" relativePath
              && relativePath != ".git"
              && !pkgs.lib.hasPrefix ".git/" relativePath;
          };
        in
        rustPlatform.buildRustPackage {
          pname = "yzs";
          version = "0.1.0";

          src = source;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "--bin"
            "yzs"
          ];
          postInstall = ''
            install -Dm644 assets/third_party/ascii_magician_1mposter.gif \
              "$out/share/yazelix_screen/ascii_magician_1mposter.gif"
          '';

          doCheck = false;

          meta = {
            description = "Standalone terminal screen animations from Yazelix";
            homepage = "https://github.com/luccahuguet/yazelix-screen";
            license = pkgs.lib.licenses.asl20;
            mainProgram = "yzs";
          };
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          yzs = yzsPackage system pkgs;
        in
        {
          default = yzs;
          yzs = yzs;
          yazelix_screen = yzs;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.yzs}/bin/yzs";
        };
        yzs = {
          type = "app";
          program = "${self.packages.${system}.yzs}/bin/yzs";
        };
        yazelix_screen = {
          type = "app";
          program = "${self.packages.${system}.yzs}/bin/yzs";
        };
      });

      checks = forAllSystems (system: {
        yzs = self.packages.${system}.yzs;
      });
    };
}
