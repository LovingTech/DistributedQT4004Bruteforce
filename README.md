# Distributed Bruteforcing tool

qt4004core is a module which has the problem to be solved. A hypothetical 4-bit processor which has 16 bytes of memory. The aim is for the program to take the most number of steps as possible for the number of bytes that the program takes.
The slave and master program are used for the distributed solving of this problem. Where master controls all the slaves connecting to it, by sending them small batches of the problem space for it to solve. The slave program is a multi-threaded program, which creates multiple threads each which then request for a batch from the master

## Buliding
To build the release version of the program run the following command.
```cargo build -r```
Then navigate to target/release and then run the program you want.
Alternatively you can use:
```cargo run --bin master/slave```
and depending on whether you put just master or just slave in the command it will start running that version.

## Invocation
### Master
The master program only has one argument which is the size of the batches which it allocates, which also has a default value.
```master 16777216```
will result in the following output.
```
Web dashboard on http://localhost:3030/metrics
Master listening on port 7878
```
### Slave
The slave program has multiple arguments. The first is the number of threads which are going to be created. This value is by default 4, however it is highly recommend, for maximum performance, that this value is equivalent to the number of cores. The other argument is the network location of the master program.
```slave 8 172.0.1.1:7878```
