- Frame : 60
- Operation per second : 500 ~ 1000 Hz

Instructions per second should be around 500 to 1000 to make it playable. So one operation takes about 16.67ms.

Timer is based on the tick, but seconds.

```
let cur_time = get_system_time();
let delta = cur_time - last_time; 
*last_time = cur_time;

accumulator += delta;
while accumulator >= 16.67ms(1/60th of a second) {
    timer();
    accumulator -= 16.67ms;
}
render();
```
