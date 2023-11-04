# Steam Query Cacher

Proxy that lazy caches steam source server queries to prevent dos using ```A2S_INFO``` attacks.

## ⚠ Disclaimer ⚠

* Only caches ```A2S_INFO```, ```A2S_PLAYER``` and ```A2S_RULES``` queries (others will get proxied without caching) with the simple response format!
* Tested on Squad dedicated servers under Windows.
* Not a ready-to-use project
* No Goldsource support!
* Hacked together tool, code quality is not good!
* No support given, feel free to contribute tho
* Code is by no way considered "clean" as it's a hacked together project to suit specific needs

## WIP/Planned

* Temporary ip blacklisting for invalid queries
* Make logs more concise as they are very messy and dont follow a clean guideline atm
