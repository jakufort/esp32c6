# Temperature probe

Uses ESP32C6 and BME280 to read temperature and humidity.

## Running

Flashing new version onto board

```shell
cargo run --release
```

**IMPORTANT**
Access to USB serial port requires for user to be in appropriate group:
- `uucp` on Arch
- `dialout` on other distros

You can add yourself to the group using

```shell
sudo usermod -a -G <group> $USER
```

Remember to log out and login after that.

