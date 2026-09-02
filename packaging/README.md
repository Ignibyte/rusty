# Packaging

`PKGBUILD` builds `rusty-git` from the main branch: the three binaries, the desktop entry
and icon, the user unit (pointed at `/usr/bin`), the key-binding and MCP snippets, the
licence and the README. It is validated with `makepkg` on the dev box and is not on the AUR
yet; that happens with the first release. `omarchy/install.sh` remains the from-source path
for a checkout.

```bash
cd packaging && makepkg -sf    # build; add -i to install
```
