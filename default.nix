{pkgs ? import (import ./npins).nixpkgs {}}: let
  mkPackage = {
    pname,
    cargoBuildFlags ? [],
  }:
    pkgs.rustPlatform.buildRustPackage {
      inherit pname cargoBuildFlags;
      cargoTestFlags = cargoBuildFlags;
      cargoHash = "sha256-raUuIdYgWC5JituToprTYVou5IFFCBAe7snLeolE6oo=";
      src = ./.;
      version = "0.1.0";
    };

  mkImage = {
    name,
    package,
    entrypoint,
  }:
    pkgs.dockerTools.buildImage {
      inherit name;
      tag = "latest";
      config = {
        Entrypoint = ["${package}/bin/${entrypoint}"];
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
    cargoBuildFlags = ["--bin" "server"];
  };

  adminPackage = mkPackage {
    pname = "cost-admin";
    cargoBuildFlags = ["--bin" "server" "--features" "admin"];
  };

  batchPackage = mkPackage {
    pname = "cost-batch";
    cargoBuildFlags = ["--bin" "batch"];
  };
in {
  default = mkImage {
    name = "cost";
    package = package;
    entrypoint = "server";
  };
  admin = mkImage {
    name = "cost-admin";
    package = adminPackage;
    entrypoint = "server";
  };
  batch = mkImage {
    name = "cost-batch";
    package = batchPackage;
    entrypoint = "batch";
  };
}
