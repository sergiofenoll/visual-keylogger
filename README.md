# visual-keylogger
Show the keys you're pressing (on your keyboard) inside a window

Currently very WIP. As-is, the program listens for all events from devices in `/dev/input/by-id/*-event-kbd`, which is a best effort guess to get devices which are keyboards (as opposed to mice, joysticks, etc.). This “heuristic” is inspired in part by [snyball/Hawck](https://github.com/snyball/Hawck/blob/master/src/KBDManager.hpp#L54).

It also uses a file watcher on the `/dev/input/by-id` directory to allow for hot-plugging keyboards. Maybe this will be changed to using `udev` at some point.

> **Note:** currently all key presses are printed to `stderr`, so don't run this anywhere you might be worried about someone snooping your passwords or something.
