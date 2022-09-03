mkdir -p ffms2_patched
if test -f "ffms2_patched/src.tar.gz"; then
    echo "already there."
else
    curl -L -o ffms2_patched/src.tar.gz https://github.com/rust-av/ffms2-rs/archive/96fcd54feeca37f8493276a95c4850ea81896905.tar.gz
fi
tar -xf ffms2_patched/src.tar.gz --strip-components=1 -C ffms2_patched/
cd ffms2_patched/ && patch -p1  < ../ffms2.patch && cd ..

