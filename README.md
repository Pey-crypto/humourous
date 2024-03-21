
# Humorous
 

## Run Instructions 
 - git clone and cd into the directory
 - run cargo run
 - then open index.html in the project directory.
## Inspiration

I initially started digging for my old java based chat app which was written in the college lab. These were based in TCP  and could only handle one client at a time. The server and client were the only users. The client would send a message to the server and the server would wait for user input and send the reply back to the client.

So I tried to mirror the behavior in TCP to web sockets. That did show me the ease of sending the input. Normally I had to convert the text to bytes and that used to lead to different problems in java. WebSockets did not have that issue, a true blessing.

reading through actix,felt like revisiting actor-model and the lifecycle. It definitely did help in reducing the LOC's. I did struggle for a while because it took time to understand how async works in rust. Then the example code for email in actix- documentation helped  a lot.

This program is still basic, requires more error handling traits.
