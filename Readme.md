# Steam Query Cacher [![build and test](https://github.com/0x280/steam-query-cacher/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/0x280/steam-query-cacher/actions/workflows/rust.yml)

Proxy that lazy caches steam source server queries to prevent dos using ```A2S_INFO``` attacks.
On windows you can set up a proxy for specific ports using the [netsh portproxy interface](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy), on linux you can use [iptables](https://serverfault.com/questions/490594/redirect-local-traffic-to-proxy-port-with-iptables).

## ⚠ Disclaimer ⚠

* Only caches ```A2S_INFO```, ```A2S_PLAYER``` and ```A2S_RULES``` queries (others will get proxied without caching) with the simple response format!
* Tested on Squad dedicated servers under Windows.
* Not a ready-to-use project
* No Goldsource support!
* No support given, feel free to contribute tho
* Code is by no way considered "clean" as it's a hacked together project to suit specific needs

## TODO

* Temporary ip blacklisting for invalid queries
* Ip based ratelimiting
* Make logs more concise as they are very messy and dont follow a clean guideline atm
* Code refactoring in general
