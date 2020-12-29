   ![](https://i.imgur.com/bS7iidD.png)
    
Lightweight, distributed, hashtable written in Rust.  Designed to be run on a cluster of AWS instances.
Implements a basic consensus protocol.  Can also be used with Docker for testing purposes.

# Usage
## Server (repeat these steps on all nodes in the cluster)
 - Set up the Rust environment if you haven't already:
	  - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Run `cargo install`
- Run `cargo run`
## Client
- Modify `client/client/iplist` to include the IP and port for each node from the steps above
- Run `cargo install`
- Run `cargo run`


# Performance
Tested on 3 AWS EC2 Free Teir instances.
![enter image description here](https://i.imgur.com/nxHxjNC.png)
