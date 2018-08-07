### TODO
- Add logging
-----

### running:
```bash
[dev@terminus arc]$ cargo run --quiet
Successfully listening on 127.0.0.1:7878
New connection with client id 2128702929 from 127.0.0.1:35710 -> 127.0.0.1:7878
Received command get foo: cid|2128702929
Sending (unformatted) response `Err("key not found")`: cid|2128702929
...
```

### client operations:
```bash
[dev@terminus arc]$ telnet 127.0.0.1 7878
Trying 127.0.0.1...
Connected to 127.0.0.1.
Escape character is '^]'.
ping
+OK
pong
get foo
-ERR
key not found
set foo bar
+OK
get foo
+OK
bar
set foo baz 
+OK
get foo
+OK
baz
del foo
+OK
get foo
-ERR
key not found
---------------------------------
sadd foo hello world 1 world
+OK
get foo
+OK
 world 1 hello
sadd foo 2
+OK
get foo
+OK
 world 2 1 hello
sadd foo world 4 
+OK
get foo
+OK
 world 2 1 4 hello
quit
+OK
disconnecting. bye!
Connection closed by foreign host.
```