#!/bin/sh
set -eux
P=$1

(
echo 'FROM sid-be:latest'
grep -E '^'$P'[[:blank:]]' dose-parse/ordered-deps \
    | cut -f2- \
    | tr ' ' '\n' | grep . | sed 's/^/RUN apt-get install -y --no-install-recommends /'
) | docker build -

