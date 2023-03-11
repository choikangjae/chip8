- Frame : 60FPS
- OpPerSec : 500HZ

Instructions per second should be between 500 to 1000 to make it playable. It will translate to one operation takes about 16.67ms.

Chip8 is the interpreter actually but we are `emulating` old machines like [COSMAC VIP] which had 1MHz CPU(1802), so to speak. Simple and easy way to solve this, sleeping a thread is a good start. But sleeping in fixed time will cause problems. To deal with it, we calculate `delta time`. 

Timer is not based on the seconds. It based on the tick.

```
let cur_time = get_system_time();
let delta = cur_time - last_time; // last_time is stored somewhere on RAM. No rules here.
*last_time = cur_time;

accumulator += delta;
while accumulator >= 16.67ms(1/60th of a second) {
    timer();
    accumulator -= 16.67ms;
}
render();
```
