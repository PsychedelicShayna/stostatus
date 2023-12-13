# STO Server Status (for Linux status bars) 
![](.github/screenshots/statusbar.png)

---

A command line utility that simply reports whether or not the Star Trek Online game server is down for maintenance or not, by querying the same API that the game's launcher does. Intended for integration with a status bar like i3blocks, i3status, xmobar, conky, etc so you can keep track of when the servers come back online, or if they're currently down for maintenance.

## Usage:
Just run it. There's literally nothing else to it. As for integrating it into you status bar, that's really up to the status bar. All this does is output the server status as regular plain text to stdout. Typically all you have to do is add a section to your status bar's config file, tell it to run this binary, give it a reasonable interval (120 seconds), and just like that, your status bar will update every 120 seconds, displaying the state of the game server. The bulk of the code is fetching the data. 

## Zero Crate Dependencies
This utility does what it does without relying on a single crated. Given how basic the needs are, attaching crates like `reqwest`, `serde`, `serde_json`, and all of _their_ dependencies to what fundamentally boils down to one simple HTTP GET request, and checking if a substring is present in the response body, just feels like laziness at the expense of bloating the binary, slowing down build times, and making offline compilation impossible without having these crates taking up space in your cache.

Think about the concept of overengineered code from the compiler's perspective; the compiler doesn't care who wrote the code or where it came from. In one scenario, it has to compile quadruple the source code, and in the other, it doesn't. You writing that source code, or it coming from a dependency tree, in and of itself, means nothing in regards to the complexity the compiler is faced with turning into machine code. Can "machine code", be overengineered, or is that a concept exclusive to source code that exists on one layer of a dependency tree it created?

In the process of working on this, I started writing my own JSON parser, until I realized that even **that** is bloat, when all I'm going to use it for is extract the value of a single uniquely identifiable key present in the response data.. Maybe I should just.. search for it..? Extracting a substring is one of the simplest possible problems in programming, why on Earth am I even writing a JSON parser? What's next, am I going to pull out a RegEx RFC? Enough.

That being said, I'm not insane. While sending an HTTP request over a TCP socket and extracting a substring are both trivial tasks, inflating GZip compressed response data is not. Thankfully, writing FFI bindings to `zlib`, a library present on practically every single Linux installation, is trivial. That means that this utility technically does have one _"dependency"_, in the same sense that `cat` may be a dependency of a shellscript; `zlib` is ubiquitous. Yet, unlike a crate, you can't even tell:


I'd be lying if I said that there isn't a "for fun" factor involved too, but I do genuinely think that some programmers are too quick to overestimate the complexity of certain tasks, and default to using a library (\*cough\*, leftpad), and that mentality can stop you from ever learning how simple it was the whole time. Seriously, making an HTTP GET request.. is just writing plaintext over a TCP socket. If you can format a string and write to a file, you can format a string and write to a socket. There's no witchcraft involved here. 

