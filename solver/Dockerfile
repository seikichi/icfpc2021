FROM public.ecr.aws/lambda/nodejs:12

RUN yum update -y && yum groupinstall -y 'Development Tools'

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH $PATH:/root/.cargo/bin
RUN rustup install stable

WORKDIR /code
COPY Cargo.lock Cargo.toml /code/
COPY src /code/src
RUN cargo build --release

COPY index.js ${LAMBDA_TASK_ROOT}

CMD ["index.handler"]
