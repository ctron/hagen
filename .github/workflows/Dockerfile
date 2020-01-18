FROM registry.access.redhat.com/ubi8/ubi-minimal

RUN mkdir /homepage
VOLUME /homepage
WORKDIR /homepage

ADD target/release/hagen /usr/local/bin

CMD /usr/local/bin/hagen
