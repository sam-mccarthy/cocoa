FROM ubuntu:latest
LABEL authors="sam"

ENTRYPOINT ["top", "-b"]