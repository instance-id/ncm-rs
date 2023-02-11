# ncm-rs

Neovim Configuration Manager (Store/Change/Swap Configurations)

I created this package because I wanted to try out Lazyvim (which is why it is referencec a few times) and other similar configuration packages and plugins without having to manually move my configuration files each time. I also wanted to be able to easily switch between configurations.

### WIP - Not completed : Do not use yet until some tests are added.

---

### Notable Features (so far)

---

- Add multiple configurations
- Conveniently switch between configurations

#### Important

---

### Make a backup of your current configuration. Always better to be safe than sorry.

To use this package, first backup your current configuration. If requested, automatic backup and movement of the original configuration could be added in the future. Unless/until that happens, you will need to move your configuration manually.

When you select a configuration using the load option, it will be symlinked to the `~/.config/nvim` directory.

<img src="media/config_backup.png">

Note, for the time being this package uses a symlink to swap/change which confuration Neovim will load.

### Example Usage

---

Add a new configuration

```bash
ncm add <name> <path> <description (optional)>
```

Load a configuration
(Once a configuration is loaded, you can use your normal `$ nvim` commands or custom keybindings as ususal)

```bash
ncm load <name>
```

<img width="800" src="https://user-images.githubusercontent.com/48368821/218245017-c2dda78f-7807-49f8-a44a-ad42d280d299.gif" />

<img width="800" src="https://user-images.githubusercontent.com/48368821/218245017-c2dda78f-7807-49f8-a44a-ad42d280d299.mp4" />

https://user-images.githubusercontent.com/48368821/218245017-c2dda78f-7807-49f8-a44a-ad42d280d299.mp4
