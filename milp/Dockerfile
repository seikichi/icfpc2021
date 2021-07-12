FROM node:12-buster
ENV FUNCTION_DIR="/function"

RUN apt-get update && \
    apt-get install -y \
    g++ \
    make \
    cmake \
    unzip \
    libcurl4-openssl-dev

RUN mkdir -p ${FUNCTION_DIR}
WORKDIR ${FUNCTION_DIR}
RUN npm install aws-lambda-ric

####################
COPY index.js ${FUNCTION_DIR}
####################

COPY ./entry_script.sh /entry_script.sh
RUN wget -P /usr/local/bin/ https://github.com/aws/aws-lambda-runtime-interface-emulator/releases/latest/download/aws-lambda-rie \
 && chmod +x /usr/local/bin/aws-lambda-rie /entry_script.sh

ENTRYPOINT ["/entry_script.sh"]
CMD ["index.handler"]
