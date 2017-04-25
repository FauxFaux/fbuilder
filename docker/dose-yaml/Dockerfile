FROM dose:latest

ARG arch

COPY $arch/packages /tmp/packages
COPY sources /tmp/sources

RUN deb-buildcheck --deb-ignore-essential --explain --successes --deb-native-arch=$arch \
        /tmp/packages /tmp/sources \
        --outfile=/tmp/buildcheck \
        || true
