# esp32c3 clock

- st7735r 128x128

## Toolchain

- rust
    - target riscv32imc-esp-espidf
    - ldproxy
    - espflash
- esp-idf v5.2.1
- python3 Pillow

## Build & Run

```
$ cargo r -r 
```

```
$ esptool  --chip esp32c3 -p /dev/ttyACM0  write_flash 0x300000 config.json
```

## screen shot

![Alt text](/screenshot/a.png)
![Alt text](/screenshot/b.png)
![Alt text](/screenshot/c.png)
![Alt text](/screenshot/d.png)

## License

MIT License
