FROM registry.access.redhat.com/ubi8/ubi-minimal

RUN mkdir /homepage
VOLUME /homepage
WORKDIR /homepage

ADD binary/hagen /usr/local/bin

CMD /usr/local/bin/hagen
