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

COPY ./entry_script.sh /entry_script.sh
RUN wget -P /usr/local/bin/ https://github.com/aws/aws-lambda-runtime-interface-emulator/releases/latest/download/aws-lambda-rie \
 && chmod +x /usr/local/bin/aws-lambda-rie /entry_script.sh

####################
WORKDIR /scip
RUN wget https://www.scipopt.org/download/release/SCIPOptSuite-7.0.3-Linux-debian.sh && chmod +x SCIPOptSuite-7.0.3-Linux-debian.sh
RUN apt-get update && apt-get install -y gcc g++ gfortran liblapack3 libtbb2 libcliquer1 libopenblas-dev libgsl23
RUN ./SCIPOptSuite-7.0.3-Linux-debian.sh --skip-license
ENV SCIPOPTDIR=/scip

RUN apt-get update && apt-get -y install python3-pip python3
RUN pip3 install pyscipopt shapely

####################
WORKDIR ${FUNCTION_DIR}
COPY new_solver.py ${FUNCTION_DIR}
COPY 12.problem ${FUNCTION_DIR}
COPY index.js ${FUNCTION_DIR}
RUN npm install aws-sdk
####################

ENTRYPOINT ["/entry_script.sh"]
CMD ["index.handler"]
