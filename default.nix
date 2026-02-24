{pkgs ? import (import ./npins).nixpkgs {}}: let
  mkPackage = {
    pname,
    cargoFlags ? [],
  }:
    pkgs.rustPlatform.buildRustPackage {
      inherit pname cargoFlags;
      cargoHash = "sha256-4LIKiCVLp5PMAkEdhPZiyKczAgR9Cfkw8mJv05yAWH4=";
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
    pname = "cost-explorer";
  };

  adminPackage = mkPackage {
    pname = "cost-explorer-admin";
    cargoFlags = ["--features" "admin"];
  };
in {
  default = mkImage {
    name = "cost-explorer";
    package = package;
  };
  admin = mkImage {
    name = "cost-explorer-admin";
    package = adminPackage;
  };
}
