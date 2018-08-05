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
get foo
-ERR
key not found
set foo 1
+OK
get foo
+OK
1
del foo
+OK
get foo
-ERR
key not found
sadd bar hello hello world 1 world 2
+OK
get bar
+OK
 1 world 2 hello
```