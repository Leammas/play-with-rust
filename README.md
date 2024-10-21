# Running

* docker run --name postgres-playground -e POSTGRES_PASSWORD=mysecretpassword -p 5432:5432 -d postgres

For http based imple
* cargo run --bin http 
* curl -X PUT localhost:3000/foo -d 'baz'
* curl -X PUT localhost:3000/fooo -d 'baz'
* curl -v localhost:3000/foo
* curl -v localhost:3000/fooo

For mini-redis impl
* cargo run --bin channelled-server
* cargo run --bin client

