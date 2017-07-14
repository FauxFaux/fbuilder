FROM sid-be:latest

## sort out locales
RUN DEBIAN_FRONTEND=noninteractive \
        apt-get install -y locales

RUN sed -i 's/# \(en_GB.UTF-8 UTF-8\)/\1/' /etc/locale.gen && \
        dpkg-reconfigure --frontend=noninteractive locales && \
        update-locale LANG=en_GB.UTF-8

ENV LANGUAGE=en_GB.UTF-8 \
    LANG=en_GB.UTF-8 \
    LC_ALL=en_GB.UTF-8

## make apt happy
RUN DEBIAN_FRONTEND=noninteractive \
        apt-get install -y apt-utils && apt-get clean

## install dev packages
RUN DEBIAN_FRONTEND=noninteractive \
        apt-get install -y devscripts git ncdu python3 vim-nox vim-scripts zsh && \
        apt-get clean && \
        chsh -s /bin/zsh

## guess some depends to cache
RUN DEBIAN_FRONTEND=noninteractive \
        apt-get install -y debhelper libssl-dev && \
        apt-get clean

WORKDIR /root

ADD home /root
