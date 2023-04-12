# Dogecoin handshaker

This program connects to chosen Dogecoin-testnet node and does the handshake process with it.

## How to use

```cargo run -- <node IP> <node port>```

example:
```cargo run -- 52.77.231.41 44556```

### Expected output
```
$ cargo run -- 52.77.231.41 44556
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
     Running `target/debug/dogecoin-handshaker 52.77.231.41 44556`
Connecting to 52.77.231.41:44556
Connection successful!
Sending version packet...
Sending version packet success! Trying to receive node version packet
Received version data from remote node: Version { header: Header { network_type: Test, command: "version", message_size: 105, hash: [130, 123, 249, 217] }, data: VersionMessageData { protocol_version: 70015, local_node_services: 5, unix_timestamp: 1681336434, node_ip_data: IpData { node_services: 0, ip_address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 123, 45, 67, 89], port: 59426 }, our_ip_data: IpData { node_services: 5, ip_address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], port: 0 }, nonce: 2905558292652613334, client_name: "/Shibetoshi:1.14.3/", node_starting_height: 4419828, relay_transactions: true } }
Sent version ack packet! Receiving version ack...
Received version ack, success! Closing...
```

## How to verify it works

Simply by running it against some existing node. Program should print information about remote node - its IP, port, supported services, client name and some other stuff.

## Areas to improve

- Add async support, currently it's single-threaded app
- Add more tests for unhappy paths
- Add tests for node connection agent
- Optimize error handling - there is some redundance in error types
- Serialization - next time I would implement this as serde module. Especially that now there is a lot of magic constants used
