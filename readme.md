## Controller

Do more computationally intensive tasks
+ Pre-generate images for display
    + remeber you want to use Floyd–Steinberg dithering
+ Poll sensor data from pico for use on system
+ Trigger neopixel animations


## front-panel

A adafruit rp pico driving sensors and displays on front panel

+ Multi-threaded to use both M0 cores
+ Worker thread for writing file to display and driving neopixel animations
+ Controlling thread for reading sensors, comunicating with other device, and sending work to worker



