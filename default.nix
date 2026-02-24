{pkgs ? import (import ./npins).nixpkgs {}}: let
  mkPackage = {
    pname,
    cargoBuildFlags ? [],
  }:
    pkgs.rustPlatform.buildRustPackage {
      inherit pname cargoBuildFlags;
      cargoTestFlags = cargoBuildFlags;
      cargoHash = "sha256-SU3rO3ObBsdtU63WlRmA8unI9dxvm1nfUbZzTC+YvYg=";
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
