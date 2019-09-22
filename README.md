# A bad audio stretcher written in Rust

## Limitations

There are only three limitations. Your input audio has to:  

* Be  wave file
* Have a sample rate of 44.1kHz
* Be mono (stereo is not supported)  

## How to use

Put a wave file you want stretched into this directory and change 
the `reader` variable in main.rs to point to this new file.  
Change the `writer` variable to point to a different file.  
Run the program. Use release mode if it's too slow.  
Turn down your volume and enjoy the bad stretch.

## What's in the box

`test.wav` contains a short YouTube clip with speech and audio. Its stretched counter-part is `stretch.wav`.  
[Source](https://www.youtube.com/watch?v=HEOz9qBG0W0)  


`test2.wav` contains the windows 98 startup sound. Its stretched counter-part is `stretch2.wav`.  
[Source](https://www.youtube.com/watch?v=tajDxBaPBBM)