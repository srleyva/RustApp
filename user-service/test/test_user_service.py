import grpc
import grpc_status
from concurrent import futures
import time

import pytest

import user_pb2
import user_pb2_grpc

# open a gRPC channel

@pytest.fixture
def client():
    channel = grpc.insecure_channel('user-service:8080')
    client = user_pb2_grpc.UserServiceStub(channel)
    new_user = user_pb2.NewUserRequest(
        user=user_pb2.User(
            first_name="test",
            last_name="test",
            username="test",
            password="password",
            uid="",
        )
    )
    try:
        client.NewUser(new_user)
    except:
        pass
    return client


@pytest.mark.parametrize("auth_request,expected_status", [
    (user_pb2.AuthRequest(username="", password="password"), grpc.StatusCode.PERMISSION_DENIED),
    (user_pb2.AuthRequest(username="test", password=""), grpc.StatusCode.PERMISSION_DENIED),
    (user_pb2.AuthRequest(username="test", password="password"), grpc.StatusCode.OK),
    (user_pb2.AuthRequest(username="test", password="bad"), grpc.StatusCode.PERMISSION_DENIED)
])
def test_authentication(client, auth_request, expected_status):
    try:
        resp = client.Auth(auth_request)
    except grpc.RpcError as ex:
        if expected_status == grpc.StatusCode.OK:
            pytest.fail(f"Exception raised where non expected: {ex}")
        assert ex.code() == expected_status

@pytest.mark.parametrize("new_user_request, expected_status", [
    (user_pb2.NewUserRequest(user=user_pb2.User(
        first_name="test",
        last_name="test",
        username="test-1",
        password="password",
        uid="")),
    grpc.StatusCode.OK),
    (user_pb2.NewUserRequest(user=user_pb2.User(
        first_name="test",
        last_name="test",
        username="test-1",
        password="password",
        uid="")),
    grpc.StatusCode.ALREADY_EXISTS),
    (user_pb2.NewUserRequest(user=user_pb2.User(
        first_name="test",
        last_name="test",
        username="",
        password="password",
        uid="")),
    grpc.StatusCode.UNAUTHENTICATED),
    (user_pb2.NewUserRequest(user=user_pb2.User(
        first_name="test",
        last_name="test",
        username="test-2",
        password="",
        uid="")),
    grpc.StatusCode.UNAUTHENTICATED),
])
def test_new_user(client, new_user_request, expected_status):
    try:
        resp = client.NewUser(new_user_request)
    except grpc.RpcError as ex:
        if expected_status == grpc.StatusCode.OK:
            pytest.fail(f"Exception raised where non expected: {ex}")
        assert ex.code() == expected_status


if __name__ == '__main__':
    time.sleep(3)
    pytest.main()