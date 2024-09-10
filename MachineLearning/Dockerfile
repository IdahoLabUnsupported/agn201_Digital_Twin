FROM rust:1.70.0 as build-rust
ADD ./*.cer /etc/ssl/certs
#ENV CARGO_HTTP_CAINFO=/etc/ssl/certs/CAINLROOT.cer

RUN mkdir /machine_learning
WORKDIR /machine_learning

COPY ./ .

RUN cargo build --release

RUN mkdir /jester
WORKDIR /jester
RUN rustup default nightly

RUN git clone https://github.com/idaholab/Jester.git /jester
RUN cargo build --release

FROM python:3.9.18 AS production
RUN mkdir /configs
RUN mkdir /linear_out
RUN mkdir /neutronics_out
RUN mkdir /anomaly_out
RUN mkdir /notebook_out

ADD ./*.cer /etc/ssl/certs

RUN python -m pip install --upgrade pip

RUN mkdir /program
WORKDIR /program

ADD python .
RUN pip install --no-cache-dir -r /program/requirements.txt
RUN pip install papermill

RUN mkdir /program/machine_learning
RUN mkdir /program/jester
COPY --from=build-rust /machine_learning/target/release /program/machine_learning
COPY --from=build-rust /jester/target/release /program/jester

RUN apt-get update && apt-get install -y supervisor
RUN mkdir -p /var/log/supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 3030
CMD ["/usr/bin/supervisord"]