{
  lib,
  stdenv,
  fetchFromGitHub,
  cmake,
  ninja,
  kdePackages,
}:

stdenv.mkDerivation {
  pname = "klassy";
  version = "6.2";

  src = fetchFromGitHub {
    owner = "paulmcauley";
    repo = "klassy";
    tag = "v6.2";
    hash = "sha256-qZ/fNYspFf0dO1pHxNnMCSB4qW4PkdzkIg4movW9obI=";
  };

  nativeBuildInputs = [
    cmake
    ninja
    kdePackages.extra-cmake-modules
    kdePackages.wrapQtAppsHook
  ];

  buildInputs = with kdePackages; [
    qtbase
    qtdeclarative
    qttools
    qtsvg
    frameworkintegration
    kcmutils
    kcolorscheme
    kconfig
    kcoreaddons
    kdecoration
    kguiaddons
    ki18n
    kiconthemes
    kirigami
    kwidgetsaddons
    kwindowsystem
  ];

  cmakeFlags = [
    (lib.cmakeBool "BUILD_QT6" true)
    (lib.cmakeBool "BUILD_QT5" false)
  ];

  meta = {
    description = "Highly customizable binary Window Decoration, Application Style and Global Theme plugin for KDE Plasma";
    homepage = "https://github.com/paulmcauley/klassy";
    platforms = lib.platforms.linux;
    license = with lib.licenses; [ bsd3 cc0 gpl2Only gpl2Plus gpl3Only gpl3Plus mit ];
    mainProgram = "klassy-settings";
  };
}
