# clink

conditional symlink farm manager

## Usage

`clink link` - creates symlinks based on Configuration

`clink unlink [-l | --leave-orphans]` - removes symlinks

## Configuration

Clink is configured in a `clink.yaml` file.

```yaml
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
```

Any directories whose name matches the format `{slug,slug,...}<any name>` get symlinked to the specified target.

**If multiple features are assigned the highest one in the config file is used.**

## Caveats

While other symlink farms like GNU stow perform "tree folding" to figure out where symlinking is most efficient,
clink only symlinks files, creating non-existing parent directories in the process. To avoid leaving
empty "orphan" folders after unlinking, clink removes empty directories up until, but not including, the
original target. This behaviour can be overwritten by specifying the --leave-orphans flag with unlink.
