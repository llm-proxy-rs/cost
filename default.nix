{pkgs ? import (import ./npins).nixpkgs {}}: let
  mkPackage = {
    pname,
    cargoBuildFlags ? [],
  }:
    pkgs.rustPlatform.buildRustPackage {
      inherit pname cargoBuildFlags;
      cargoTestFlags = cargoBuildFlags;
      cargoHash = "sha256-a1Ox7FlDf3R/fNQPrM6Czo+5txW0KaFhe9lpN8jP+bE=";
      src = ./.;
      version = "0.1.0";
    };

  mkImage = {
    name,
    package,
  }:
    pkgs.dockerTools.buildImage {
      inherit name;
      tag = "latest";
      config = {
        Entrypoint = ["${package}/bin/server"];
      };
      copyToRoot = pkgs.buildEnv {
        name = "image-root";
        paths = [
          pkgs.dockerTools.caCertificates
        ];
      };
    };

  package = mkPackage {
    pname = "cost";
  };

  adminPackage = mkPackage {
    pname = "cost-admin";
    cargoBuildFlags = ["--features" "admin"];
  };
in {
  default = mkImage {
    name = "cost";
    package = package;
  };
  admin = mkImage {
    name = "cost-admin";
    package = adminPackage;
  };
}
