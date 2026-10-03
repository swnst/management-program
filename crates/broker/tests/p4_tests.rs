use broker_proto::{
    deserialize_msg, serialize_msg, BrokerRequest, BrokerResponse,
};

#[test]
fn test_broker_request_serialization() {
    let req = BrokerRequest::PurgeStandbyList;
    let bytes = serialize_msg(&req).expect("serialize request");
    let decoded: BrokerRequest = deserialize_msg(&bytes).expect("deserialize request");
    match decoded {
        BrokerRequest::PurgeStandbyList => {}
        _ => panic!("Unexpected request variant"),
    }
}

#[test]
fn test_broker_response_serialization() {
    let resp = BrokerResponse::Success;
    let bytes = serialize_msg(&resp).expect("serialize response");
    let decoded: BrokerResponse = deserialize_msg(&bytes).expect("deserialize response");
    match decoded {
        BrokerResponse::Success => {}
        _ => panic!("Unexpected response variant"),
    }

    let err_resp = BrokerResponse::Error("Access Denied".into());
    let bytes2 = serialize_msg(&err_resp).expect("serialize err");
    let decoded2: BrokerResponse = deserialize_msg(&bytes2).expect("deserialize err");
    match decoded2 {
        BrokerResponse::Error(msg) => assert_eq!(msg, "Access Denied"),
        _ => panic!("Unexpected response variant"),
    }
}

#[test]
fn test_ipc_named_pipe_roundtrip() {
    use broker::{BrokerClient, BrokerServer};
    use std::thread;
    use std::time::Duration;

    let test_pipe_name = r"\\.\pipe\LumenBrokerTestPipe_P4";
    let server = BrokerServer::new(Some(test_pipe_name));

    let server_handle = thread::spawn(move || {
        let _ = server.run();
    });

    // Give server time to listen
    thread::sleep(Duration::from_millis(100));

    let client = BrokerClient::connect_with_name(test_pipe_name).expect("connect to test pipe");

    // Ping check
    let pong = client
        .send_request(&BrokerRequest::Ping)
        .expect("send ping");
    match pong {
        BrokerResponse::Pong => {}
        _ => panic!("Expected Pong"),
    }

    // Graceful shutdown
    let shutdown = client
        .send_request(&BrokerRequest::Shutdown)
        .expect("send shutdown");
    match shutdown {
        BrokerResponse::Success => {}
        _ => panic!("Expected Success on shutdown"),
    }

    let _ = server_handle.join();
}
