# Passenger Frequency Measurement

Source code and config files for Medium articles about *Passenger Frequency Measurement*:

* [Part I: Technical background and setup of Paxcounter for passenger frequency measurement](https://medium.com/research-center-kodis/passenger-frequency-measurement-part-i-6a75d2aa63fd)
* Part II: Pull and save passenger frequency data
* Part III: Visualize passenger frequency data (coming soon)

## Structure
* `connector`: Source code for data connector (written in Rust)
* `database`: Init file for creating database (streaming database RisingWave)
* `dashboard`: Source code for dashboard (Plotly Dash)

## Config Paxcounter
`platform.ini`
```TOML
; MODIFY FOLLOWING LINES
[board]
halfile = lopy4.h
 
[env]
board = lopy4
 
; ADD FOLLOWING LINES
[env:lopy4]
platform = espressif32
board = lopy4
 
; change microcontroller
board_build.mcu = esp32
 
; change MCU frequency
board_build.f_cpu = 240000000L
 
upload_protocol = esptool
```

`ota.conf`
```c++
...
// MAC sniffing settings
#define BLECOUNTER                      1       // set to 0 if you do not want to start the BLE sniffer
#define WIFICOUNTER                     1       // set to 0 if you do not want to start the WIFI sniffer
#define RSSILIMIT                       0       // 0...-128, set to 0 if you do not want to filter signals
 
// Payload send cycle and encoding
#define SENDCYCLE                       30      // payload send cycle [seconds/2], 0 .. 255
...
```

`lopy4.h`
```c++
...
// select WIFI antenna (internal = onboard / external = u.fl socket)
#define HAS_ANTENNA_SWITCH  (21) // pin for switching wifi antenna (P12)
#define WIFI_ANTENNA 0    // 0 = internal, 1 = external
 
// uncomment defines in this section ONLY if your LoPy lives on a EXPANSION BOARD
#define HAS_LED (12) // use if LoPy is on Expansion Board, this has a user LED
#define LED_ACTIVE_LOW 1 // use if LoPy is on Expansion Board, this has a user LED
...
```
