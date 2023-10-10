# clink [conditional symlink farm]

`clink` is a conditional symlink farm manager. You can define features as described in [Configuration](#configuration), 
with a condition on whether directories tagged with this feature should be symlinked. This condition can 
either be `all`, the wanted OS (`macos` | `linux`) or a custom command.

## Usage

`clink link` - creates symlinks based on `clink.yaml` in current directory

`clink unlink [-l | --leave-orphans]` - removes symlinks

## Configuration

Clink is configured in a `clink.yaml` file.

```yaml
ignore:     # ignore file or folder names, applies to all directories and subdirectories
  - .DS_Store

features:
  - slug: all
    enabled: all # all, macos, linux or custom command
    target: ~/   # location to symlink this feature to

  - slug: mac
    enabled: macos
    target: ~/

  - slug: mac-opt
    enabled: macos
    target: /opt/

  - slug: linux
    enabled: linux
    target: ~/

  - slug: custom
    # custom enabled commands have to be tagged with !command
    # the specified command needs to return exit code 0 to count as enabled
    enabled: !command /some/custom/script
    target: ~/
    
  - slug: linux_wayland
    target: ~/
    enabled: !command /bin/bash -c "if [[ $XDG_SESSION_TYPE -ne wayland ]] ; then exit 1 ; fi"
```

The content of directories matching the format `{slug,slug,...}<name>`, where at least one enabled feature
matches a specified slug, gets symlinked to the target specified in the feature.

**If multiple features are assigned, priority is determined by their order in the config file. (top - higher priority, bottom - lower priority)**

## Caveats

While other symlink farms like GNU stow perform "tree folding" to figure out where symlinking is most efficient,
clink only symlinks files, creating non-existing parent directories in the process. This is somewhat mitigated
by the default unlink behaviour, where clink removes all empty parent folders of a link, up until the target itself.

You can disable this by adding the `-l (--leave-orphans)` flag to unlink.
