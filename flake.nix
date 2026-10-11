{
  description = "rune: safe Rust OCI runtime with deterministic policy and Linux adapters";
  inputs.nixpkgs.url = "https://channels.nixos.org/nixos-26.05/nixexprs.tar.xz";
  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      each = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs { inherit system; };
      src = nixpkgs.lib.fileset.toSource {
        root = ./.;
        fileset = nixpkgs.lib.fileset.unions [
          ./Cargo.toml ./Cargo.lock ./src ./crates ./tests ./vendor ./scripts
        ];
      };
      packageFor = system:
        let pkgs = pkgsFor system;
        in pkgs.rustPlatform.buildRustPackage {
          pname = "rune-oci";
          version = "0.1.0";
          inherit src;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.libseccomp ];
          strictDeps = true;
          doCheck = true;
          nativeCheckInputs = [ pkgs.coreutils ];
          cargoBuildFlags = [ "--workspace" "--locked" "--bin" "rune" ];
          cargoTestFlags = [ "--workspace" "--locked" ];
          postInstall = ''test -x "$out/bin/rune"'';
          meta = {
            description = "Experimental Rust OCI runtime with native libcontainer adapter";
            license = pkgs.lib.licenses.mit;
            mainProgram = "rune";
            platforms = systems;
          };
        };
    in {
      packages = each (system: { default = packageFor system; });
      apps = each (system: { default = { type = "app"; program = "${self.packages.${system}.default}/bin/rune"; }; });
      checks = each (system: { package = self.packages.${system}.default; });
      devShells = each (system:
        let pkgs = pkgsFor system;
        in { default = pkgs.mkShell {
          packages = with pkgs; [ cargo rustc rustfmt clippy rust-analyzer libseccomp pkg-config python3 git podman ];
        }; });
    };
}
