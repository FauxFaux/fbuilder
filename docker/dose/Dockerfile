FROM sid-be:latest

RUN apt-get install --no-install-recommends -y \
    cppo \
    ocaml-nox \
    ocamlbuild \
    libcudf-ocaml-dev \
    libocamlgraph-ocaml-dev \
    libre-ocaml-dev

COPY dose /tmp/dose

RUN cd /tmp/dose && \
    ./configure && \
    make && \
    make install && \
    rm -rf /tmp/dose
