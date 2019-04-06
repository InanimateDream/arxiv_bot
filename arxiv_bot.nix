with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "cargo-env";
    buildInputs = [ pkgconfig openssl sqlite ];
    shellHook = ''
        PATH="$PATH:~/.cargo/bin"
    '';
}
