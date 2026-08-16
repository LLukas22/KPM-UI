{
  description = "KPM UI development and Kindle release environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      target = "arm-kindlehf-linux-gnueabihf";
      rustTarget = "arm-unknown-linux-gnueabihf";

      koxtoolchain = pkgs.stdenvNoCC.mkDerivation {
        pname = "kindlehf-koxtoolchain";
        version = "2026.04";

        src = pkgs.fetchurl {
          url = "https://github.com/KindleModding/koxtoolchain/releases/download/2026.04/kindlehf.tar.gz";
          hash = "sha256-0I06XLuqGEzAzCJ53x1kpxZWqyweAiPK4yS40wyNc+A=";
        };

        nativeBuildInputs = [ pkgs.autoPatchelfHook ];
        buildInputs = [ pkgs.stdenv.cc.cc.lib ];
        dontConfigure = true;
        dontBuild = true;

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -a "${target}/." "$out/"
          patchShebangs "$out"
          runHook postInstall
        '';
      };

      kindletool = pkgs.stdenv.mkDerivation {
        pname = "kindletool";
        version = "2026-06-22";

        src = pkgs.fetchFromGitHub {
          owner = "KindleModding";
          repo = "KindleTool";
          rev = "4d559f6c5b3ebf7dd2d5cfb26c7fd9a601234eda";
          hash = "sha256-5Hm74+DXGpig9V5v82KF5mkWtwHXnNAWUK9s3kjTUXU=";
        };

        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs; [
          libarchive
          nettle
          zlib
        ];

        postPatch = ''
          substituteInPlace KindleTool/kindle_tool.c \
            --replace-fail 'if (freopen(NULL, "rb", stdin) == NULL)' 'if (0)' \
            --replace-fail 'if (freopen(NULL, "wb", stdout) == NULL)' 'if (0)'
        '';

        buildPhase = ''
          runHook preBuild
          make -C KindleTool CFLAGS="-O2 -pipe" KT_NO_USERATHOST_TAG=1
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          install -Dm755 KindleTool/Release/kindletool "$out/bin/kindletool"
          runHook postInstall
        '';
      };

      sdkSource = pkgs.fetchFromGitHub {
        owner = "KindleModding";
        repo = "kindle-sdk";
        rev = "b4a6c99d718a7cf74935f36105c62491b4336a61";
        hash = "sha256-xhtqr4fSGllNCi8vsosSU8AzVnVBm1iXrHbh6hzE/fM=";
      };

      cjsonSource = pkgs.fetchFromGitHub {
        owner = "DaveGamble";
        repo = "cJSON";
        rev = "fb16e5cf358798aabb049655975cde8427101056";
        hash = "sha256-ODOwUVlRzDtUrqb2za9fkytp//hYwf48Uc5TMWwLiBo=";
      };

      curlSource = pkgs.fetchFromGitHub {
        owner = "curl";
        repo = "curl";
        rev = "cd95ee9f771361acf241629d2fe5507e308082a2";
        hash = "sha256-EEBRMdJrDkFL9Ol00hobmLouUBwdLEZPUEetVjIjXno=";
      };

      openlipcSource = pkgs.fetchFromGitHub {
        owner = "arkq";
        repo = "openlipc";
        rev = "91645e77b291d92926b4292e31b2fa470322d1c7";
        hash = "sha256-9dUD6gmZwzkWZOPM2A9sh2liK+SKq3nYHxoUZ56IHi8=";
      };

      paperwhiteFirmware = pkgs.fetchurl {
        url = "https://s3.amazonaws.com/firmwaredownloads/update_kindle_all_new_paperwhite_v2_5.16.3.bin";
        hash = "sha256-K84tLtjN0rJQr0l6YDRZIau7Q7sgSDY6ebrHSPxX/tk=";
      };

      scribeFirmware = pkgs.fetchurl {
        url = "https://s3.amazonaws.com/firmwaredownloads/update_kindle_scribe_5.16.3.bin";
        hash = "sha256-jzXKmR6hIa6HbsWq4x6nqyZMDYNKEKwEjQpvo95iG0Q=";
      };

      kindleSdk = pkgs.stdenvNoCC.mkDerivation {
        pname = "kindlehf-sdk";
        version = "2026-06-22";
        src = sdkSource;

        nativeBuildInputs = with pkgs; [
          e2fsprogs
          findutils
          gzip
          kindletool
          pkg-config
          rsync
        ];

        dontConfigure = true;
        dontBuild = true;
        dontFixup = true;

        installPhase = ''
          runHook preInstall

          cp -a ${koxtoolchain} "$out"
          chmod -R u+w "$out"
          sysroot="$out/${target}/sysroot"

          mkdir -p modules/cJSON modules/curl modules/openlipc
          cp -a ${cjsonSource}/. modules/cJSON/
          cp -a ${curlSource}/. modules/curl/
          cp -a ${openlipcSource}/. modules/openlipc/

          extract_firmware() {
            firmware="$1"
            destination="$2"
            mkdir -p "$destination/update" "$destination/root/usr"
            ${kindletool}/bin/kindletool extract "$firmware" "$destination/update"
            gzip -dc "$destination/update/rootfs.img.gz" > "$destination/rootfs.img"
            debugfs -R "rdump /usr/lib $destination/root/usr" "$destination/rootfs.img"
            debugfs -R "rdump /lib $destination/root" "$destination/rootfs.img"
          }

          extract_firmware ${paperwhiteFirmware} firmware-paperwhite
          extract_firmware ${scribeFirmware} firmware-scribe

          rm -rf "$sysroot/usr/lib/pkgconfig"
          mkdir -p patch/any/usr/lib/pkgconfig patch/kindlehf/usr/lib/pkgconfig
          cp -a pkgconfig/any/. patch/any/usr/lib/pkgconfig/
          cp -a pkgconfig/kindlehf/. patch/kindlehf/usr/lib/pkgconfig/
          find patch -path '*/pkgconfig/*.pc' -type f -exec \
            sed -i 's@%TARGET%@${target}@g' {} +

          cp modules/openlipc/include/openlipc.h patch/any/usr/include/lipc.h
          cp modules/cJSON/cJSON.h modules/cJSON/cJSON_Utils.h patch/any/usr/include/
          rm -rf patch/any/usr/include/curl
          cp -a modules/curl/include/curl patch/any/usr/include/curl

          cp -a patch/any/. "$sysroot/"
          cp -a patch/kindlehf/. "$sysroot/"

          rsync -a --ignore-existing firmware-paperwhite/root/usr/lib/ "$sysroot/usr/lib/"
          rsync -a --ignore-existing firmware-paperwhite/root/lib/ "$sysroot/lib/"
          rsync -a --ignore-existing firmware-scribe/root/usr/lib/ "$sysroot/usr/lib/"
          rsync -a --ignore-existing firmware-scribe/root/lib/ "$sysroot/lib/"

          while IFS= read -r link; do
            destination="$(readlink "$link")"
            case "$destination" in
              /*) ln -sfn "$sysroot$destination" "$link" ;;
            esac
          done < <(find "$sysroot/lib" "$sysroot/usr/lib" -type l)

          chmod -R a-w "$out"
          runHook postInstall
        '';

        preferLocalBuild = true;
        allowSubstitutes = false;
      };

      rustHostBundle = pkgs.fetchurl {
        url = "https://static.rust-lang.org/dist/2026-07-16/rust-1.97.1-x86_64-unknown-linux-gnu.tar.gz";
        hash = "sha256-tM28fMaw7gomZrGHJ2n9sq2Dk7KLY5UvZJO0tADkgys=";
      };

      rustTargetBundle = pkgs.fetchurl {
        url = "https://static.rust-lang.org/dist/2026-07-16/rust-std-1.97.1-${rustTarget}.tar.gz";
        hash = "sha256-Q1994jUToEiuBqkfavU5nSKopT97CUgtg2OyLO9iCiA=";
      };

      rustToolchain = pkgs.stdenv.mkDerivation {
        pname = "rust-kindlehf-toolchain";
        version = "1.97.1";
        dontUnpack = true;
        nativeBuildInputs = [ pkgs.autoPatchelfHook ];
        buildInputs = with pkgs; [
          stdenv.cc.cc.lib
          zlib
        ];

        installPhase = ''
          runHook preInstall
          mkdir host target
          tar -xzf ${rustHostBundle} -C host --strip-components=1
          tar -xzf ${rustTargetBundle} -C target --strip-components=1
          patchShebangs host/install.sh target/install.sh
          host/install.sh --prefix="$out" \
            --components="rustc,cargo,rust-std-x86_64-unknown-linux-gnu"
          target/install.sh --prefix="$out" --components="rust-std-${rustTarget}"
          runHook postInstall
        '';

        dontStrip = true;
      };

      sysroot = "${kindleSdk}/${target}/sysroot";
    in
    {
      devShells.${system} = {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            just
            rustc
            rustfmt
          ];
        };

        desktop = pkgs.mkShell {
          packages = with pkgs; [
            bubblewrap
            cargo
            git
            gtk2
            just
            meson
            ninja
            pkg-config
            rustc
            sqlite
          ];
        };

        kindle = pkgs.mkShell {
          packages = with pkgs; [
            binutils
            glib.bin
            gvfs
            just
            kindleSdk
            openssh
            python3
            rustToolchain
            usbutils
          ];

          shellHook = ''
            export KINDLEHF_SDK=${kindleSdk}
            export KINDLEHF_SYSROOT=${sysroot}
            export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=${kindleSdk}/bin/${target}-gcc
            export PKG_CONFIG_ALLOW_CROSS=1
            export PKG_CONFIG_SYSROOT_DIR=${sysroot}
            export PKG_CONFIG_LIBDIR=${sysroot}/usr/lib/pkgconfig
          '';
        };
      };
    };
}
