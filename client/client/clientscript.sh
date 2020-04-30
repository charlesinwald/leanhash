./../../stop.sh #1>/dev/null 2>/dev/null
./../../startnodes.sh 1>/dev/null 2>/dev/null
cargo run -- --operations 1000 --max_key 10
./../../stop.sh #1>/dev/null 2>/dev/null
./../../startnodes.sh 1>/dev/null 2>/dev/null
cargo run -- --operations 1000 --max_key 100
./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
 cargo run -- --operations 1000 --max_key 1000
 ./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
 cargo run -- --operations 1000 --max_key 10000
./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
 cargo run -- --operations 5000 --max_key 10
./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
cargo run -- --operations 5000 --max_key 100
./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
  cargo run -- --operations 5000 --max_key 1000
  ./../../stop.sh #1>/dev/null 2>/dev/null
 ./../../startnodes.sh 1>/dev/null 2>/dev/null
  cargo run -- --operations 5000 --max_key 10000
