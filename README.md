# minecraft_reverse_proxy
A Rust implementation of a reverse proxy, used for exposing a Minecraft server run on a friend's computer behind a firewall.

## Principle

```
[============ MC server host ============]         [=== reverse proxy ===]         [=== MC clients ===]
          25565                              25566                         25565   
MC server <------ minecraft_server_exposer ------> minecraft_reverse_proxy <------ MC clients
```

Notice how the guy **hosting** the Minecraft server **only has outgoing connections** and no incoming ones!

## Setup

1. The guy hosting the reverse proxy has to open ports `25565` and `25566`, then simply start the `minecraft_reverse_proxy`.
2. The guy hosting the minecraft server has to do no firewall changes at all, simply start the `minecraft_server_exposer` (and tell it the IP address of the reverse proxy).
