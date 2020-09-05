FROM python:3.8

WORKDIR /tests

ADD requirements.txt .
RUN pip3 install -r requirements.txt

ADD user_pb2.py  .
ADD user_pb2_grpc.py .
ADD test_user_service.py .

RUN apt install coreutils

RUN which sleep

ENTRYPOINT ["python3", "test_user_service.py"]