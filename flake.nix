{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          # --- Rust backend toolchain ---
          pkg-config
          cargo
          rustc
          rust-analyzer
          # aws-lc-rs / rustls native build needs cmake + a C/clang toolchain
          cmake
          clang

          # --- Frontend toolchain ---
          # nodejs_22 bundles node + npm + npx. npm handles the frontend
          # dependencies itself: `cd frontend && npm ci && npm run dev`.
          nodejs_22

          # --- Containers / devops ---
          # Podman runs rootless without a daemon, so it works in this dev
          # shell out of the box. `podman-compose` reads the docker-compose.*.yml
          # files. (Docker CLI/compose kept for those who prefer a Docker daemon.)
          podman
          podman-compose
          docker
          docker-compose
        ];
        buildInputs = with pkgs; [
          openssl
          gtk4
          libadwaita
        ];
      };
    };
}
